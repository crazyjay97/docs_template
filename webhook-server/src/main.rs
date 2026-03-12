//! GitHub Webhook Server for auto-deployment
//!
//! Verification mechanisms:
//! 1. HMAC SHA256 signature verification (WEBHOOK_SECRET)
//! 2. GitHub official IP range verification
//! 3. Organization/user whitelist verification

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
use std::net::SocketAddr;
use std::process::Stdio;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::{error, info};

type HmacSha256 = Hmac<Sha256>;

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
    source_dir: String,      // Directory to clone repositories
    deploy_dir: String,      // Directory to deploy dist output
    skip_ip_check: bool,
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
            deploy_dir: env::var("DEPLOY_DIR")
                .unwrap_or_else(|_| "/var/www/docs".to_string()),
            skip_ip_check: env::var("SKIP_IP_CHECK")
                .unwrap_or_default()
                .parse()
                .unwrap_or(false),
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

/// GitHub webhook payload
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
    owner: Option<OwnerInfo>,
}

#[derive(Deserialize, Debug)]
struct OwnerInfo {
    login: String,
    #[serde(rename = "type")]
    owner_type: Option<String>,
}

#[derive(Deserialize, Debug)]
struct CommitInfo {
    committer: Option<Committer>,
}

#[derive(Deserialize, Debug)]
struct Committer {
    name: String,
}

#[derive(Deserialize)]
struct PingPayload {
    zen: Option<String>,
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
    // Check organization whitelist
    if owner_type == Some("Organization") {
        if !allowed_orgs.is_empty() {
            return allowed_orgs.iter().any(|o| o == owner_login);
        }
    }
    // Check user whitelist
    if owner_type == Some("User") || owner_type.is_none() {
        if !allowed_users.is_empty() {
            return allowed_users.iter().any(|u| u == owner_login);
        }
    }

    // If the corresponding whitelist is empty, allow
    (owner_type == Some("Organization") && allowed_orgs.is_empty())
        || (owner_type != Some("Organization") && allowed_users.is_empty())
}

/// Execute deployment: git clone/pull, pip install, make dist, copy dist
async fn run_deployment(
    source_dir: &str,
    deploy_dir: &str,
    repo_full_name: &str,
    repo_name: &str,
) -> Result<String, String> {
    let repo_path = format!("{}/{}", source_dir.trim_end_matches('/'), repo_name);
    let repo_url = format!("https://github.com/{}.git", repo_full_name);

    // Check if repository already exists
    let repo_exists = tokio::fs::try_exists(&repo_path).await.unwrap_or(false);

    if repo_exists {
        // Pull latest changes
        info!("Repository {} exists, pulling latest changes...", repo_name);
        let output = Command::new("git")
            .arg("pull")
            .current_dir(&repo_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run git pull: {}", e))?;

        if !output.status.success() {
            return Err(format!("git pull failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        info!("git pull completed for {}", repo_name);
    } else {
        // Clone repository
        info!("Cloning repository {}...", repo_url);

        // Ensure source directory exists
        tokio::fs::create_dir_all(source_dir)
            .await
            .map_err(|e| format!("Failed to create source directory: {}", e))?;

        let output = Command::new("git")
            .arg("clone")
            .arg(&repo_url)
            .arg(&repo_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run git clone: {}", e))?;

        if !output.status.success() {
            return Err(format!("git clone failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        info!("git clone completed for {}", repo_name);
    }

    // Find docs folder
    let docs_path = format!("{}/docs", repo_path);
    let docs_exists = tokio::fs::try_exists(&docs_path).await.unwrap_or(false);

    if !docs_exists {
        return Err(format!("docs folder not found in {}", repo_name));
    }

    // Install pip dependencies
    info!("Installing pip dependencies for {}...", repo_name);
    let requirements_path = format!("{}/requirements.txt", docs_path);
    let requirements_exists = tokio::fs::try_exists(&requirements_path).await.unwrap_or(false);

    if requirements_exists {
        let output = Command::new("pip")
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
            return Err(format!("pip install failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        info!("pip install completed for {}", repo_name);
    } else {
        info!("No requirements.txt found, skipping pip install for {}", repo_name);
    }

    // Run make dist
    info!("Running make dist for {}...", repo_name);
    let output = Command::new("make")
        .arg("dist")
        .current_dir(&docs_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run make dist: {}", e))?;

    if !output.status.success() {
        return Err(format!("make dist failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    info!("make dist completed for {}", repo_name);

    // Copy dist to deploy directory
    let dist_path = format!("{}/dist", docs_path);
    let target_dir = format!("{}/{}", deploy_dir.trim_end_matches('/'), repo_name);

    info!("Copying dist to {}...", target_dir);

    // Ensure deploy directory exists
    tokio::fs::create_dir_all(&target_dir)
        .await
        .map_err(|e| format!("Failed to create deploy directory: {}", e))?;

    // Copy dist contents to deploy directory
    copy_directory(&dist_path, &target_dir)
        .map_err(|e| format!("Failed to copy dist: {}", e))?;

    info!("Deployment completed for {} to {}", repo_name, target_dir);
    Ok(format!("Successfully deployed {} to {}", repo_name, target_dir))
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
        "service": "github-webhook-server"
    }))
}

/// GitHub webhook endpoint
async fn webhook(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> StatusCode {
    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let delivery_id = headers
        .get("X-GitHub-Delivery")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    // Get client IP - try X-Forwarded-For first (for reverse proxy), then fall back to direct connection
    let client_addr = headers
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<SocketAddr>().ok())
        .unwrap_or(addr);

    info!(
        "Received webhook event: {}, delivery: {}, from: {}",
        event_type, delivery_id, client_addr
    );

    // 1. IP address verification (optional skip)
    if !state.config.skip_ip_check && !is_github_ip(client_addr, &state.github_ips) {
        info!("Webhook from non-GitHub IP: {}", client_addr);
        return StatusCode::FORBIDDEN;
    }

    // Handle ping event
    if event_type == "ping" {
        if let Ok(payload) = serde_json::from_slice::<PingPayload>(&body) {
            info!("Ping event received: {:?}", payload.zen);
        }
        return StatusCode::OK;
    }

    // Only handle push events
    if event_type != "push" {
        info!("Ignoring non-push event: {}", event_type);
        return StatusCode::OK;
    }

    // 2. Signature verification
    if !state.config.webhook_secret.is_empty()
        && !verify_signature(&body, signature, &state.config.webhook_secret)
    {
        info!("Webhook signature verification failed");
        return StatusCode::FORBIDDEN;
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
        .map(|o| o.login.as_str())
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
        .map(|c| c.name.as_str())
        .unwrap_or("unknown")
        .to_string();

    info!("Push to {} in {} by {}", branch, repo_full_name, committer);

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

        let (status, message) = match run_deployment(
            &config.source_dir,
            &config.deploy_dir,
            &repo_full_name,
            repo_name,
        ).await {
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
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("webhook_server=info".parse().unwrap()),
        )
        .init();

    // Load .env file
    let _ = dotenvy::dotenv();

    let config = Arc::new(AppConfig::default());
    let deployment_log = Arc::new(RwLock::new(Vec::new()));

    // Parse GitHub IPs
    let github_ips: Vec<IpNetwork> = GITHUB_WEBHOOK_IPS
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect();

    let state = AppState {
        config,
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

    // Build router
    let app = Router::new()
        .route("/webhook", post(webhook))
        .route("/health", get(health_check))
        .route("/logs", get(get_logs))
        .with_state(state);

    info!("Starting webhook server on port {}", port);
    info!("Source directory: {}", source_dir);
    info!("Deploy directory: {}", deploy_dir);
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

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind to port");

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("Failed to start server");
}
