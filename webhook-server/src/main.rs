//! Git provider webhook server for auto-deployment
//!
//! Verification mechanisms:
//! 1. GitHub HMAC SHA256 signature verification (WEBHOOK_SECRET)
//! 2. Gitee token verification (WEBHOOK_SECRET)
//! 3. GitHub official IP range verification
//! 4. Organization/user whitelist verification

use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use hex::ToHex;
use hmac::{Hmac, Mac};
use ipnetwork::IpNetwork;
use serde::Deserialize;
use sha2::Sha256;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::process::Stdio;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::{self, Duration};
use tracing::{error, info, warn, Level};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WebhookProvider {
    GitHub,
    Gitee,
}

impl WebhookProvider {
    fn as_str(self) -> &'static str {
        match self {
            Self::GitHub => "GitHub",
            Self::Gitee => "Gitee",
        }
    }
}

/// GitHub Webhook IP ranges (periodically updated: https://api.github.com/meta)
const GITHUB_WEBHOOK_IPS: &[&str] = &[
    "192.30.252.0/22",
    "185.199.108.0/22",
    "140.82.112.0/20",
    "143.55.64.0/20",
    "2a0a:a440::/29",
    "2606:50c0::/32",
];

/// Application state
#[derive(Clone)]
struct AppState {
    config: Arc<AppConfig>,
    deployment_log: Arc<RwLock<Vec<DeploymentLog>>>,
    github_ips: Arc<Vec<IpNetwork>>,
}

/// Configuration
struct AppConfig {
    webhook_secret: String,
    port: u16,
    allowed_orgs: Vec<String>,
    allowed_users: Vec<String>,
    source_dir: String, // Directory to clone repositories
    deploy_dir: String, // Directory to deploy dist output
    skip_ip_check: bool,
    log_file_path: String, // Log file path
    use_venv: bool,        // Whether to use a Python virtual environment
    venv_dir: String,      // Virtual environment directory (relative to docs/)
    python_bin: String,    // Python executable used to create venv
    upgrade_pip: bool,     // Whether to upgrade pip inside the venv
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            webhook_secret: env::var("WEBHOOK_SECRET").unwrap_or_default(),
            port: env::var("PORT")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            allowed_orgs: env::var("ALLOWED_ORGS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            allowed_users: env::var("ALLOWED_USERS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            source_dir: env::var("SOURCE_DIR")
                .unwrap_or_else(|_| "/var/www/docs-source".to_string()),
            deploy_dir: env::var("DEPLOY_DIR").unwrap_or_else(|_| "/var/www/docs".to_string()),
            skip_ip_check: env::var("SKIP_IP_CHECK")
                .unwrap_or_default()
                .parse()
                .unwrap_or(false),
            log_file_path: env::var("LOG_FILE_PATH")
                .unwrap_or_else(|_| "webhook-server.log".to_string()),
            use_venv: env::var("USE_VENV")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            venv_dir: env::var("VENV_DIR").unwrap_or_else(|_| "venv".to_string()),
            python_bin: env::var("PYTHON_BIN").unwrap_or_else(|_| "python3".to_string()),
            upgrade_pip: env::var("UPGRADE_PIP")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}

/// Deployment log entry
#[derive(Clone, Deserialize, serde::Serialize)]
struct DeploymentLog {
    timestamp: String,
    status: String,
    message: String,
    branch: Option<String>,
    committer: Option<String>,
    repo: Option<String>,
}

/// Git provider webhook payload
#[derive(Deserialize, Debug)]
struct WebhookPayload {
    #[serde(rename = "ref")]
    ref_name: Option<String>,
    repository: Option<RepositoryInfo>,
    head_commit: Option<CommitInfo>,
}

#[derive(Deserialize, Debug)]
struct RepositoryInfo {
    full_name: String,
    clone_url: Option<String>,
    git_http_url: Option<String>,
    owner: Option<OwnerInfo>,
}

#[derive(Deserialize, Debug)]
struct OwnerInfo {
    login: Option<String>,
    name: Option<String>,
    path: Option<String>,
    #[serde(rename = "type")]
    owner_type: Option<String>,
}

#[derive(Deserialize, Debug)]
struct CommitInfo {
    committer: Option<Committer>,
}

#[derive(Deserialize, Debug)]
struct Committer {
    name: Option<String>,
}

#[derive(Deserialize)]
struct PingPayload {
    zen: Option<String>,
}

#[derive(Deserialize)]
struct GiteeTestPayload {
    hook_name: Option<String>,
}

/// Verify GitHub webhook signature
fn verify_signature(payload: &[u8], signature: &str, secret: &str) -> bool {
    if secret.is_empty() || signature.is_empty() {
        return false;
    }

    let expected_signature = format!("sha256={}", {
        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
        mac.update(payload);
        mac.finalize().into_bytes().encode_hex::<String>()
    });

    ConstantTimeEq::ct_eq(expected_signature.as_bytes(), signature.as_bytes()).into()
}

fn verify_token(token: &str, secret: &str) -> bool {
    if secret.is_empty() || token.is_empty() {
        return false;
    }

    ConstantTimeEq::ct_eq(token.as_bytes(), secret.as_bytes()).into()
}

fn detect_provider(headers: &HeaderMap) -> Option<WebhookProvider> {
    if headers.contains_key("X-Gitee-Event") || headers.contains_key("X-Gitee-Token") {
        return Some(WebhookProvider::Gitee);
    }

    if headers.contains_key("X-GitHub-Event") || headers.contains_key("X-Hub-Signature-256") {
        return Some(WebhookProvider::GitHub);
    }

    None
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> &'a str {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
}

fn get_client_addr(headers: &HeaderMap, fallback: SocketAddr) -> SocketAddr {
    headers
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .and_then(|s| {
            s.parse::<SocketAddr>().ok().or_else(|| {
                s.parse::<IpAddr>()
                    .ok()
                    .map(|ip| SocketAddr::new(ip, fallback.port()))
            })
        })
        .unwrap_or(fallback)
}

fn is_push_event(provider: WebhookProvider, event_type: &str) -> bool {
    match provider {
        WebhookProvider::GitHub => event_type == "push",
        WebhookProvider::Gitee => {
            event_type.eq_ignore_ascii_case("Push Hook") || event_type.eq_ignore_ascii_case("push")
        }
    }
}

fn repo_clone_url(
    provider: WebhookProvider,
    repository: Option<&RepositoryInfo>,
    repo_full_name: &str,
) -> String {
    repository
        .and_then(|r| r.clone_url.as_deref().or(r.git_http_url.as_deref()))
        .map(str::to_string)
        .unwrap_or_else(|| match provider {
            WebhookProvider::GitHub => format!("https://github.com/{}.git", repo_full_name),
            WebhookProvider::Gitee => format!("https://gitee.com/{}.git", repo_full_name),
        })
}

/// Check if IP is from GitHub
fn is_github_ip(addr: SocketAddr, github_ips: &[IpNetwork]) -> bool {
    let ip = addr.ip();
    github_ips.iter().any(|net| net.contains(ip))
}

/// Verify owner (org or user) is in whitelist
fn is_allowed_owner(
    owner_login: &str,
    owner_type: Option<&str>,
    allowed_orgs: &[String],
    allowed_users: &[String],
) -> bool {
    if allowed_orgs.is_empty() && allowed_users.is_empty() {
        return true;
    }

    let owner_type = owner_type.unwrap_or_default();
    let is_org_like = matches!(owner_type, "Organization" | "Group" | "Enterprise");
    let is_user_like = owner_type.is_empty() || owner_type == "User";

    if is_org_like {
        return allowed_orgs.iter().any(|o| o == owner_login);
    }

    if is_user_like {
        return allowed_users.iter().any(|u| u == owner_login)
            || allowed_orgs.iter().any(|o| o == owner_login);
    }

    allowed_users.iter().any(|u| u == owner_login) || allowed_orgs.iter().any(|o| o == owner_login)
}

/// Retry configuration
const MAX_RETRIES: u32 = 2;
const RETRY_DELAY_SECS: u64 = 300; // 5 minutes

/// Venv-related settings forwarded to the deployment routine.
struct VenvSettings<'a> {
    use_venv: bool,
    venv_dir: &'a str,
    python_bin: &'a str,
    upgrade_pip: bool,
}

/// Execute deployment: git clone/pull, pip install, make dist, copy dist
/// Returns (success, attempts_made)
async fn run_deployment(
    source_dir: &str,
    deploy_dir: &str,
    repo_url: &str,
    repo_name: &str,
    branch_ref: &str,
    venv: &VenvSettings<'_>,
) -> Result<String, String> {
    let mut last_error: Option<String> = None;

    for attempt in 0..=MAX_RETRIES {
        info!(
            "Deployment attempt {} of {} for {}",
            attempt + 1,
            MAX_RETRIES + 1,
            repo_name
        );

        match run_deployment_once(
            source_dir, deploy_dir, repo_url, repo_name, branch_ref, venv,
        )
        .await
        {
            Ok(output) => {
                info!(
                    "Deployment succeeded on attempt {} for {}",
                    attempt + 1,
                    repo_name
                );
                return Ok(output);
            }
            Err(e) => {
                error!(
                    "Deployment attempt {} failed for {}: {}",
                    attempt + 1,
                    repo_name,
                    e
                );
                last_error = Some(e);

                if attempt < MAX_RETRIES {
                    info!(
                        "Retrying deployment for {} in {} seconds...",
                        repo_name, RETRY_DELAY_SECS
                    );
                    time::sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "Deployment failed after all retries".to_string()))
}

/// Ensure a Python virtual environment exists at `venv_path`, creating it with
/// `python_bin -m venv` if missing. If `venv` fails because `ensurepip` is
/// unavailable (common on Debian/Ubuntu without `python3-venv`), fall back to
/// creating it without pip and bootstrap pip via `get-pip.py`.
async fn ensure_venv(docs_path: &str, venv_dir: &str, python_bin: &str) -> Result<String, String> {
    let venv_path = format!("{}/{}", docs_path.trim_end_matches('/'), venv_dir);

    // A venv is only reusable when both `python` and `python3` symlinks resolve
    // to real files. A previous failed `ensurepip` run can leave the dir with
    // broken symlinks; `tokio::fs::metadata` follows symlinks so it returns
    // an error for broken ones, which is what we want here.
    let python_in_venv = format!("{}/bin/python", venv_path);
    let python3_in_venv = format!("{}/bin/python3", venv_path);
    let usable = tokio::fs::metadata(&python_in_venv).await.is_ok()
        && tokio::fs::metadata(&python3_in_venv).await.is_ok();

    if usable {
        info!("Using existing virtual environment at {}", venv_path);
        return Ok(venv_path);
    }

    // Remove any partial/broken venv so the next create call starts clean —
    // otherwise `python3 -m venv` reuses the broken state and keeps failing.
    if tokio::fs::try_exists(&venv_path).await.unwrap_or(false) {
        info!(
            "Removing incomplete venv at {} before recreating",
            venv_path
        );
        tokio::fs::remove_dir_all(&venv_path)
            .await
            .map_err(|e| format!("Failed to remove incomplete venv at {}: {}", venv_path, e))?;
    }

    info!(
        "Creating virtual environment at {} using {}...",
        venv_path, python_bin
    );

    let output = Command::new(python_bin)
        .arg("-m")
        .arg("venv")
        .arg(&venv_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run `{} -m venv`: {}", python_bin, e))?;

    if output.status.success() {
        return Ok(venv_path);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    info!(
        "`{} -m venv` failed, retrying without pip: {}",
        python_bin, stderr
    );

    // The failed attempt likely left a partial venv; wipe it before the fallback.
    if tokio::fs::try_exists(&venv_path).await.unwrap_or(false) {
        tokio::fs::remove_dir_all(&venv_path)
            .await
            .map_err(|e| format!("Failed to remove failed venv at {}: {}", venv_path, e))?;
    }

    // Fallback: create venv without pip, then bootstrap pip manually.
    let output = Command::new(python_bin)
        .arg("-m")
        .arg("venv")
        .arg("--without-pip")
        .arg(&venv_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| {
            format!(
                "Failed to run `{} -m venv --without-pip`: {}",
                python_bin, e
            )
        })?;

    if !output.status.success() {
        return Err(format!(
            "Failed to create virtual environment: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(venv_path)
}

/// Ensure pip is installed inside the given venv. Tries `ensurepip` first, and
/// falls back to downloading `get-pip.py` from pypa when it's unavailable.
/// When `upgrade` is true, pip/setuptools/wheel are upgraded after install.
async fn ensure_pip(venv_path: &str, upgrade: bool) -> Result<(), String> {
    let pip_bin = format!("{}/bin/pip", venv_path);
    let python_bin = format!("{}/bin/python", venv_path);

    let pip_exists = tokio::fs::try_exists(&pip_bin).await.unwrap_or(false);

    if !pip_exists {
        info!("pip not found in venv, attempting ensurepip...");

        let output = Command::new(&python_bin)
            .arg("-m")
            .arg("ensurepip")
            .arg("--upgrade")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run ensurepip: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            info!(
                "ensurepip failed, falling back to get-pip.py: {}",
                err.trim()
            );

            // Download get-pip.py to the venv directory and run it.
            let get_pip_path = format!("{}/get-pip.py", venv_path);
            info!("Downloading get-pip.py...");
            let resp = reqwest::get("https://bootstrap.pypa.io/get-pip.py")
                .await
                .map_err(|e| format!("Failed to download get-pip.py: {}", e))?;
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| format!("Failed to read get-pip.py body: {}", e))?;
            tokio::fs::write(&get_pip_path, &bytes)
                .await
                .map_err(|e| format!("Failed to write get-pip.py: {}", e))?;

            let output = Command::new(&python_bin)
                .arg(&get_pip_path)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
                .map_err(|e| format!("Failed to run get-pip.py: {}", e))?;

            // Best-effort cleanup; ignore errors.
            let _ = tokio::fs::remove_file(&get_pip_path).await;

            if !output.status.success() {
                return Err(format!(
                    "get-pip.py failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
    }

    if upgrade {
        info!("Upgrading pip/setuptools/wheel inside venv...");
        let output = Command::new(&python_bin)
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("--upgrade")
            .arg("pip")
            .arg("setuptools")
            .arg("wheel")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to upgrade pip: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "pip upgrade failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }

    Ok(())
}

/// Build a PATH value that places the venv's bin dir ahead of the inherited PATH,
/// so that subprocesses (make, sphinx-build, etc.) pick up venv tools first.
fn venv_path_env(venv_path: &str) -> String {
    let venv_bin = format!("{}/bin", venv_path.trim_end_matches('/'));
    match env::var("PATH") {
        Ok(existing) if !existing.is_empty() => format!("{}:{}", venv_bin, existing),
        _ => venv_bin,
    }
}

/// Run a command while streaming stdout/stderr into the service logs.
async fn run_logged_command(mut command: Command, label: &str) -> Result<(), String> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run {}: {}", label, e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| format!("Failed to capture {} stdout", label))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| format!("Failed to capture {} stderr", label))?;

    let stdout_label = label.to_string();
    let stdout_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| format!("Failed reading {} stdout: {}", stdout_label, e))?
        {
            info!("[{} stdout] {}", stdout_label, line);
        }
        Ok::<(), String>(())
    });

    let stderr_label = label.to_string();
    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| format!("Failed reading {} stderr: {}", stderr_label, e))?
        {
            warn!("[{} stderr] {}", stderr_label, line);
        }
        Ok::<(), String>(())
    });

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed waiting for {}: {}", label, e))?;

    stdout_task
        .await
        .map_err(|e| format!("{} stdout task failed: {}", label, e))??;
    stderr_task
        .await
        .map_err(|e| format!("{} stderr task failed: {}", label, e))??;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{} failed with status {}", label, status))
    }
}

/// Single deployment attempt (without retry logic)
async fn run_deployment_once(
    source_dir: &str,
    deploy_dir: &str,
    repo_url: &str,
    repo_name: &str,
    branch_ref: &str,
    venv: &VenvSettings<'_>,
) -> Result<String, String> {
    let repo_path = format!("{}/{}", source_dir.trim_end_matches('/'), repo_name);
    let branch_name = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);

    // Check if repository already exists
    let repo_exists = tokio::fs::try_exists(&repo_path).await.unwrap_or(false);

    if repo_exists {
        // Force local checkout to match the remote branch exactly.
        info!(
            "Repository {} exists, syncing to remote branch {}...",
            repo_name, branch_name
        );
        let output = Command::new("git")
            .arg("fetch")
            .arg("origin")
            .arg(branch_name)
            .current_dir(&repo_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run git fetch: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "git fetch failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let output = Command::new("git")
            .arg("checkout")
            .arg("-B")
            .arg(branch_name)
            .arg(format!("origin/{}", branch_name))
            .current_dir(&repo_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run git checkout: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "git checkout failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let output = Command::new("git")
            .arg("reset")
            .arg("--hard")
            .arg(format!("origin/{}", branch_name))
            .current_dir(&repo_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run git reset --hard: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "git reset --hard failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let output = Command::new("git")
            .arg("clean")
            .arg("-fd")
            .current_dir(&repo_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run git clean: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "git clean failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        info!(
            "Repository {} synced to origin/{} successfully",
            repo_name, branch_name
        );
    } else {
        // Clone repository
        info!("Cloning repository {}...", repo_url);

        // Ensure source directory exists
        tokio::fs::create_dir_all(source_dir)
            .await
            .map_err(|e| format!("Failed to create source directory: {}", e))?;

        let output = Command::new("git")
            .arg("clone")
            .arg("--branch")
            .arg(branch_name)
            .arg(&repo_url)
            .arg(&repo_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run git clone: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "git clone failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        info!("git clone completed for {}", repo_name);
    }

    // Find docs folder
    let docs_path = format!("{}/docs", repo_path);
    let docs_exists = tokio::fs::try_exists(&docs_path).await.unwrap_or(false);

    if !docs_exists {
        return Err(format!("docs folder not found in {}", repo_name));
    }

    // Prepare virtual environment (optional) and pip
    let venv_path_opt = if venv.use_venv {
        let path = ensure_venv(&docs_path, venv.venv_dir, venv.python_bin).await?;
        ensure_pip(&path, venv.upgrade_pip).await?;
        Some(path)
    } else {
        None
    };

    // Install pip dependencies
    info!("Installing pip dependencies for {}...", repo_name);
    let requirements_path = format!("{}/requirements.txt", docs_path);
    let requirements_exists = tokio::fs::try_exists(&requirements_path)
        .await
        .unwrap_or(false);

    if requirements_exists {
        let (pip_program, pip_first_args): (String, Vec<String>) = match &venv_path_opt {
            Some(path) => (
                format!("{}/bin/python", path),
                vec!["-m".to_string(), "pip".to_string()],
            ),
            None => ("pip".to_string(), Vec::new()),
        };

        let mut cmd = Command::new(&pip_program);
        for a in &pip_first_args {
            cmd.arg(a);
        }
        let output = cmd
            .arg("install")
            .arg("-r")
            .arg(&requirements_path)
            .current_dir(&docs_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run pip install: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "pip install failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        info!("pip install completed for {}", repo_name);
    } else {
        info!(
            "No requirements.txt found, skipping pip install for {}",
            repo_name
        );
    }

    // Compute PATH to be used by subsequent make/find commands so the venv's
    // bin dir (sphinx-build etc.) is picked up first when using a venv.
    let path_env = venv_path_opt.as_deref().map(venv_path_env);

    // Ensure build helper scripts are executable before running make.
    info!(
        "Ensuring helper scripts are executable for {}...",
        repo_name
    );
    let output = Command::new("find")
        .arg(".")
        .arg("(")
        .arg("-path")
        .arg("./scripts/*")
        .arg("-o")
        .arg("-name")
        .arg("*.sh")
        .arg(")")
        .arg("-type")
        .arg("f")
        .arg("-exec")
        .arg("chmod")
        .arg("+x")
        .arg("{}")
        .arg("+")
        .current_dir(&docs_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to chmod helper scripts: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "chmod helper scripts failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    info!("Helper scripts are executable for {}", repo_name);

    // Run make clean
    info!("Running make clean for {}...", repo_name);
    let mut make_clean = Command::new("make");
    make_clean.arg("clean").current_dir(&docs_path);
    if let Some(path_env) = &path_env {
        make_clean.env("PATH", path_env);
    }
    let output = make_clean
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run make clean: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "make clean failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    info!("make clean completed for {}", repo_name);

    // Run make dist
    info!("Running make dist for {}...", repo_name);
    let mut make_dist = Command::new("make");
    make_dist.arg("dist").current_dir(&docs_path);
    if let Some(path_env) = &path_env {
        make_dist.env("PATH", path_env);
    }
    run_logged_command(make_dist, "make dist").await?;
    info!("make dist completed for {}", repo_name);

    // Copy dist to deploy directory
    let dist_path = format!("{}/dist", docs_path);
    let target_dir = format!("{}/{}", deploy_dir.trim_end_matches('/'), repo_name);

    info!("Copying dist to {}...", target_dir);

    // Remove existing target directory to ensure clean deployment
    // This prevents stale files from previous builds
    tokio::fs::remove_dir_all(&target_dir).await.unwrap_or(());

    // Ensure deploy directory exists
    tokio::fs::create_dir_all(&target_dir)
        .await
        .map_err(|e| format!("Failed to create deploy directory: {}", e))?;

    // Copy dist contents to deploy directory
    copy_directory(&dist_path, &target_dir).map_err(|e| format!("Failed to copy dist: {}", e))?;

    info!("Deployment completed for {} to {}", repo_name, target_dir);
    Ok(format!(
        "Successfully deployed {} to {}",
        repo_name, target_dir
    ))
}

/// Recursively copy directory contents
fn copy_directory(src: &str, dst: &str) -> std::io::Result<()> {
    use std::fs;

    // Create destination directory if it doesn't exist
    fs::create_dir_all(dst)?;

    // Read source directory entries
    let entries = fs::read_dir(src)?;

    for entry in entries {
        let entry = entry?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = std::path::Path::new(dst).join(&file_name);

        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            // Recursively copy subdirectory
            copy_directory(src_path.to_str().unwrap(), dst_path.to_str().unwrap())?;
        } else {
            // Copy file
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "git-webhook-server"
    }))
}

/// Git provider webhook endpoint
async fn webhook(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> StatusCode {
    let provider = match detect_provider(&headers) {
        Some(p) => p,
        None => {
            info!("Webhook provider could not be detected");
            return StatusCode::BAD_REQUEST;
        }
    };

    let signature = header_value(&headers, "X-Hub-Signature-256");
    let gitee_token = header_value(&headers, "X-Gitee-Token");
    let event_type = match provider {
        WebhookProvider::GitHub => header_value(&headers, "X-GitHub-Event"),
        WebhookProvider::Gitee => header_value(&headers, "X-Gitee-Event"),
    };
    let delivery_id = match provider {
        WebhookProvider::GitHub => header_value(&headers, "X-GitHub-Delivery"),
        WebhookProvider::Gitee => header_value(&headers, "X-Gitee-Timestamp"),
    };
    let delivery_id = if delivery_id.is_empty() {
        "unknown"
    } else {
        delivery_id
    };

    // Get client IP - try X-Forwarded-For first (for reverse proxy), then fall back to direct connection.
    let client_addr = get_client_addr(&headers, addr);

    info!(
        "Received {} webhook event: {}, delivery: {}, from: {}",
        provider.as_str(),
        event_type,
        delivery_id,
        client_addr
    );

    // 1. IP address verification. Gitee does not use the GitHub IP whitelist.
    if provider == WebhookProvider::GitHub
        && !state.config.skip_ip_check
        && !is_github_ip(client_addr, &state.github_ips)
    {
        info!("Webhook from non-GitHub IP: {}", client_addr);
        return StatusCode::FORBIDDEN;
    }

    // Handle ping event
    if provider == WebhookProvider::GitHub && event_type == "ping" {
        if let Ok(payload) = serde_json::from_slice::<PingPayload>(&body) {
            info!("GitHub ping event received: {:?}", payload.zen);
        }
        return StatusCode::OK;
    }

    if provider == WebhookProvider::Gitee && event_type.eq_ignore_ascii_case("Test Hook") {
        if let Ok(payload) = serde_json::from_slice::<GiteeTestPayload>(&body) {
            info!("Gitee test event received: {:?}", payload.hook_name);
        }
        return StatusCode::OK;
    }

    // Only handle push events.
    if !is_push_event(provider, event_type) {
        info!(
            "Ignoring non-push {} event: {}",
            provider.as_str(),
            event_type
        );
        return StatusCode::OK;
    }

    // 2. Provider-specific secret verification.
    if !state.config.webhook_secret.is_empty() {
        let verified = match provider {
            WebhookProvider::GitHub => {
                verify_signature(&body, signature, &state.config.webhook_secret)
            }
            WebhookProvider::Gitee => verify_token(gitee_token, &state.config.webhook_secret),
        };

        if !verified {
            info!("{} webhook secret verification failed", provider.as_str());
            return StatusCode::FORBIDDEN;
        }
    }

    // Parse payload
    let payload: WebhookPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to parse payload: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    let branch = payload.ref_name.as_deref().unwrap_or("unknown").to_string();
    let repo_full_name = payload
        .repository
        .as_ref()
        .map(|r| r.full_name.as_str())
        .unwrap_or("unknown")
        .to_string();

    let repo_owner_login = payload
        .repository
        .as_ref()
        .and_then(|r| r.owner.as_ref())
        .and_then(|o| {
            o.login
                .as_deref()
                .or(o.name.as_deref())
                .or(o.path.as_deref())
        })
        .unwrap_or("unknown")
        .to_string();

    let repo_owner_type = payload
        .repository
        .as_ref()
        .and_then(|r| r.owner.as_ref())
        .and_then(|o| o.owner_type.as_deref());

    let committer = payload
        .head_commit
        .as_ref()
        .and_then(|c| c.committer.as_ref())
        .and_then(|c| c.name.as_deref())
        .unwrap_or("unknown")
        .to_string();
    let repo_url = repo_clone_url(provider, payload.repository.as_ref(), &repo_full_name);

    info!(
        "{} push to {} in {} by {}",
        provider.as_str(),
        branch,
        repo_full_name,
        committer
    );

    // 3. Organization/user whitelist verification
    if !is_allowed_owner(
        &repo_owner_login,
        repo_owner_type,
        &state.config.allowed_orgs,
        &state.config.allowed_users,
    ) {
        info!(
            "Webhook from unauthorized owner: {} (type: {:?})",
            repo_owner_login, repo_owner_type
        );
        return StatusCode::FORBIDDEN;
    }

    // Only deploy for main/master branch
    if !["refs/heads/main", "refs/heads/master"].contains(&branch.as_str()) {
        info!("Ignoring push to non-main branch: {}", branch);
        return StatusCode::OK;
    }

    // Trigger deployment in background
    let config = state.config.clone();
    let log_clone = state.deployment_log.clone();

    tokio::spawn(async move {
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Extract repo name from repo_full_name (e.g., "owner/repo" -> "repo")
        let repo_name = repo_full_name.split('/').last().unwrap_or(&repo_full_name);

        let venv_settings = VenvSettings {
            use_venv: config.use_venv,
            venv_dir: &config.venv_dir,
            python_bin: &config.python_bin,
            upgrade_pip: config.upgrade_pip,
        };

        let (status, message) = match run_deployment(
            &config.source_dir,
            &config.deploy_dir,
            &repo_url,
            repo_name,
            &branch,
            &venv_settings,
        )
        .await
        {
            Ok(output) => {
                info!("Deployment completed successfully");
                ("success".to_string(), output)
            }
            Err(e) => {
                error!("Deployment failed: {}", e);
                ("failed".to_string(), e)
            }
        };

        let log_entry = DeploymentLog {
            timestamp,
            status,
            message: message.chars().take(500).collect(),
            branch: Some(branch),
            committer: Some(committer),
            repo: Some(repo_full_name),
        };

        let mut log = log_clone.write().await;
        log.push(log_entry);

        // Keep only last 100 entries
        let len = log.len();
        if len > 100 {
            let _ = log.split_off(len - 100);
        }
    });

    StatusCode::OK
}

/// Get deployment logs endpoint
#[axum::debug_handler]
async fn get_logs(State(state): State<AppState>) -> Json<Vec<DeploymentLog>> {
    let logs = state.deployment_log.read().await;
    Json(logs.clone())
}

#[tokio::main]
async fn main() {
    // Load .env file first
    let _ = dotenvy::dotenv();

    let config = AppConfig::default();

    // Setup file appender for logging
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        env::current_dir().unwrap_or_default(),
        &config.log_file_path,
    );

    // Initialize tracing with both file and console output
    let file_layer = Layer::new()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_file(true)
        .with_line_number(true);

    let console_layer = Layer::new()
        .with_ansi(true)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("webhook_server", Level::INFO)
                .with_target("tokio", Level::WARN)
                .with_target("axum", Level::WARN),
        )
        .with(file_layer)
        .with(console_layer)
        .init();

    let config = Arc::new(config);
    let deployment_log = Arc::new(RwLock::new(Vec::new()));

    // Parse GitHub IPs
    let github_ips: Vec<IpNetwork> = GITHUB_WEBHOOK_IPS
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect();

    let state = AppState {
        config: config.clone(),
        deployment_log,
        github_ips: Arc::new(github_ips),
    };

    let port = state.config.port;
    let source_dir = state.config.source_dir.clone();
    let deploy_dir = state.config.deploy_dir.clone();
    let webhook_secret = state.config.webhook_secret.clone();
    let allowed_orgs = state.config.allowed_orgs.clone();
    let allowed_users = state.config.allowed_users.clone();
    let skip_ip_check = state.config.skip_ip_check;
    let log_file_path = state.config.log_file_path.clone();
    let use_venv = state.config.use_venv;
    let venv_dir = state.config.venv_dir.clone();
    let python_bin = state.config.python_bin.clone();
    let upgrade_pip = state.config.upgrade_pip;

    // Build router
    let app = Router::new()
        .route("/webhook", post(webhook))
        .route("/health", get(health_check))
        .route("/logs", get(get_logs))
        .with_state(state);

    info!("Starting webhook server on port {}", port);
    info!("Source directory: {}", source_dir);
    info!("Deploy directory: {}", deploy_dir);
    info!("Log file: {}", log_file_path);
    info!(
        "Webhook secret configured: {}",
        if webhook_secret.is_empty() {
            "No"
        } else {
            "Yes"
        }
    );
    info!("Allowed orgs: {:?}", allowed_orgs);
    info!("Allowed users: {:?}", allowed_users);
    info!(
        "IP check: {}",
        if skip_ip_check { "disabled" } else { "enabled" }
    );
    info!(
        "Use venv: {} (dir: {}, python: {}, upgrade_pip: {})",
        use_venv, venv_dir, python_bin, upgrade_pip
    );

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind to port");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Failed to start server");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gitee_push_event_is_supported() {
        assert!(is_push_event(WebhookProvider::Gitee, "Push Hook"));
        assert!(is_push_event(WebhookProvider::Gitee, "push"));
        assert!(!is_push_event(WebhookProvider::Gitee, "Merge Request Hook"));
    }

    #[test]
    fn gitee_token_uses_webhook_secret() {
        assert!(verify_token("secret", "secret"));
        assert!(!verify_token("wrong", "secret"));
        assert!(!verify_token("", "secret"));
        assert!(!verify_token("secret", ""));
    }

    #[test]
    fn whitelist_accepts_gitee_group_like_owner() {
        let allowed_orgs = vec!["docs-team".to_string()];
        let allowed_users = Vec::new();

        assert!(is_allowed_owner(
            "docs-team",
            Some("Group"),
            &allowed_orgs,
            &allowed_users
        ));
        assert!(!is_allowed_owner(
            "other-team",
            Some("Group"),
            &allowed_orgs,
            &allowed_users
        ));
    }

    #[test]
    fn whitelist_checks_unknown_owner_type_against_both_lists() {
        let allowed_orgs = vec!["docs-team".to_string()];
        let allowed_users = vec!["alice".to_string()];

        assert!(is_allowed_owner(
            "docs-team",
            None,
            &allowed_orgs,
            &allowed_users
        ));
        assert!(is_allowed_owner(
            "alice",
            None,
            &allowed_orgs,
            &allowed_users
        ));
        assert!(!is_allowed_owner(
            "bob",
            None,
            &allowed_orgs,
            &allowed_users
        ));
    }
}
