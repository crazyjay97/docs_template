//! GitHub Webhook Server for auto-deployment
//!
//! 验证机制:
//! 1. HMAC SHA256 签名验证 (WEBHOOK_SECRET)
//! 2. GitHub 官方 IP 段验证
//! 3. 组织/仓库白名单验证

use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
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
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

type HmacSha256 = Hmac<Sha256>;

/// GitHub Webhook IP 段 (定期更新：https://api.github.com/meta)
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
    deploy_script: String,
    port: u16,
    allowed_orgs: Vec<String>,
    allowed_repos: Vec<String>,
    skip_ip_check: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            webhook_secret: env::var("WEBHOOK_SECRET").unwrap_or_default(),
            deploy_script: env::var("DEPLOY_SCRIPT").unwrap_or_else(|_| "./deploy.sh".to_string()),
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
            allowed_repos: env::var("ALLOWED_REPOS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            skip_ip_check: env::var("SKIP_IP_CHECK")
                .unwrap_or_default()
                .parse()
                .unwrap_or(false),
        }
    }
}

/// Deployment log entry
#[derive(Clone, Deserialize)]
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
    sender: Option<SenderInfo>,
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
    message: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Committer {
    name: String,
}

#[derive(Deserialize, Debug)]
struct SenderInfo {
    login: String,
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

    hmac::digest::ConstantTimeEq::ct_eq(expected_signature.as_bytes(), signature.as_bytes()).into()
}

/// Check if IP is from GitHub
fn is_github_ip(addr: SocketAddr, github_ips: &[IpNetwork]) -> bool {
    let ip = addr.ip();
    github_ips.iter().any(|net| net.contains(ip))
}

/// Verify repository is in whitelist
fn is_allowed_repo(repo_full_name: &str, org: &str, allowed_orgs: &[String], allowed_repos: &[String]) -> bool {
    // 检查组织是否在白名单
    if !allowed_orgs.is_empty() {
        let repo_org = repo_full_name.split('/').next().unwrap_or("");
        if !allowed_orgs.iter().any(|o| o == repo_org) {
            return false;
        }
    }

    // 检查具体仓库是否在白名单
    if !allowed_repos.is_empty() {
        return allowed_repos.iter().any(|r| r == repo_full_name);
    }

    // 如果白名单都为空，则允许
    allowed_orgs.is_empty()
}

/// Execute deployment script
async fn run_deployment(script: &str) -> Result<String, String> {
    let output = Command::new("/bin/bash")
        .arg("-c")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to execute deploy script: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(stdout.to_string())
    } else {
        Err(format!("{}{}", stdout, stderr))
    }
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
    axum::http::Headers(headers): axum::http::Headers,
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

    info!(
        "Received webhook event: {}, delivery: {}, from: {}",
        event_type, delivery_id, addr
    );

    // 1. IP 地址验证 (可选跳过)
    if !state.config.skip_ip_check && !is_github_ip(addr, &state.github_ips) {
        warn!("Webhook from non-GitHub IP: {}", addr);
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

    // 2. 签名验证
    if !state.config.webhook_secret.is_empty()
        && !verify_signature(&body, signature, &state.config.webhook_secret)
    {
        warn!("Webhook signature verification failed");
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

    let branch = payload.ref_name.as_deref().unwrap_or("unknown");
    let repo_full_name = payload
        .repository
        .as_ref()
        .map(|r| r.full_name.as_str())
        .unwrap_or("unknown");

    let repo_owner = payload
        .repository
        .as_ref()
        .and_then(|r| r.owner.as_ref())
        .map(|o| o.login.as_str())
        .unwrap_or("unknown");

    let committer = payload
        .head_commit
        .as_ref()
        .and_then(|c| c.committer.as_ref())
        .map(|c| c.name.as_str())
        .unwrap_or("unknown");

    info!("Push to {} in {} by {}", branch, repo_full_name, committer);

    // 3. 组织/仓库白名单验证
    if !is_allowed_repo(
        repo_full_name,
        repo_owner,
        &state.config.allowed_orgs,
        &state.config.allowed_repos
    ) {
        warn!(
            "Webhook from unauthorized repository: {} (owner: {})",
            repo_full_name, repo_owner
        );
        return StatusCode::FORBIDDEN;
    }

    // Only deploy for main/master branch
    if !["refs/heads/main", "refs/heads/master"].contains(&branch) {
        info!("Ignoring push to non-main branch: {}", branch);
        return StatusCode::OK;
    }

    // Trigger deployment in background
    let config = state.config.clone();
    let log_clone = state.deployment_log.clone();

    tokio::spawn(async move {
        let timestamp = chrono::Utc::now().to_rfc3339();

        let (status, message) = match run_deployment(&config.deploy_script).await {
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
            branch: Some(branch.to_string()),
            committer: Some(committer.to_string()),
            repo: Some(repo_full_name.to_string()),
        };

        let mut log = log_clone.write().await;
        log.push(log_entry);

        // Keep only last 100 entries
        if log.len() > 100 {
            *log = log.split_off(log.len() - 100);
        }
    });

    StatusCode::OK
}

/// Get deployment logs endpoint
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

    // Build router
    let app = Router::new()
        .route("/webhook", post(webhook))
        .route("/health", get(health_check))
        .route("/logs", get(get_logs))
        .with_state(state);

    let port = state.config.port;

    info!("Starting webhook server on port {}", port);
    info!("Deploy script: {}", state.config.deploy_script);
    info!(
        "Webhook secret configured: {}",
        if state.config.webhook_secret.is_empty() {
            "No"
        } else {
            "Yes"
        }
    );
    info!("Allowed orgs: {:?}", state.config.allowed_orgs);
    info!("Allowed repos: {:?}", state.config.allowed_repos);
    info!(
        "IP check: {}",
        if state.config.skip_ip_check { "disabled" } else { "enabled" }
    );

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind to port");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
