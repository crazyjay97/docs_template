use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use hex::ToHex;
use hmac::{Hmac, Mac};
use ipnetwork::IpNetwork;
use serde::Deserialize;
use sha2::Sha256;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::config::AppConfig;
use crate::deploy::{self, VenvSettings};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebhookProvider {
    GitHub,
    Gitee,
    GitLab,
}

impl WebhookProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GitHub => "GitHub",
            Self::Gitee => "Gitee",
            Self::GitLab => "GitLab",
        }
    }
}

pub const GITHUB_WEBHOOK_IPS: &[&str] = &[
    "192.30.252.0/22",
    "185.199.108.0/22",
    "140.82.112.0/20",
    "143.55.64.0/20",
    "2a0a:a440::/29",
    "2606:50c0::/32",
];

#[derive(Clone)]
pub struct WebhookState {
    pub config: Arc<AppConfig>,
    pub deployment_log: Arc<RwLock<Vec<DeploymentLog>>>,
    pub github_ips: Arc<Vec<IpNetwork>>,
}

#[derive(Clone, Deserialize, serde::Serialize)]
pub struct DeploymentLog {
    pub timestamp: String,
    pub status: String,
    pub message: String,
    pub branch: Option<String>,
    pub committer: Option<String>,
    pub repo: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct WebhookPayload {
    #[serde(rename = "ref")]
    ref_name: Option<String>,
    repository: Option<RepositoryInfo>,
    project: Option<ProjectInfo>,
    head_commit: Option<CommitInfo>,
    user_name: Option<String>,
}

#[derive(Deserialize, Debug)]
struct RepositoryInfo {
    full_name: Option<String>,
    name: Option<String>,
    clone_url: Option<String>,
    git_http_url: Option<String>,
    owner: Option<OwnerInfo>,
}

#[derive(Deserialize, Debug)]
struct ProjectInfo {
    path_with_namespace: Option<String>,
    name: Option<String>,
    git_http_url: Option<String>,
    http_url_to_repo: Option<String>,
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

/// PLACEHOLDER_WEBHOOK_FUNCTIONS

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
    if headers.contains_key("X-Gitlab-Event") || headers.contains_key("X-Gitlab-Token") {
        return Some(WebhookProvider::GitLab);
    }
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
        WebhookProvider::Gitee | WebhookProvider::GitLab => {
            event_type.eq_ignore_ascii_case("Push Hook") || event_type.eq_ignore_ascii_case("push")
        }
    }
}

fn payload_repo_full_name(payload: &WebhookPayload) -> String {
    payload.repository.as_ref().and_then(|r| r.full_name.as_deref())
        .or_else(|| payload.project.as_ref().and_then(|p| p.path_with_namespace.as_deref()))
        .or_else(|| payload.repository.as_ref().and_then(|r| r.name.as_deref()))
        .or_else(|| payload.project.as_ref().and_then(|p| p.name.as_deref()))
        .unwrap_or("unknown")
        .to_string()
}

fn payload_owner_login(payload: &WebhookPayload) -> String {
    if let Some(owner) = payload.repository.as_ref().and_then(|r| r.owner.as_ref()) {
        if let Some(login) = owner.login.as_deref().or(owner.name.as_deref()).or(owner.path.as_deref()) {
            return login.to_string();
        }
    }
    payload.project.as_ref()
        .and_then(|p| p.path_with_namespace.as_deref())
        .and_then(|path| path.rsplit_once('/').map(|(ns, _)| ns))
        .unwrap_or("unknown")
        .to_string()
}

fn payload_owner_type(payload: &WebhookPayload) -> Option<&str> {
    payload.repository.as_ref()
        .and_then(|r| r.owner.as_ref())
        .and_then(|o| o.owner_type.as_deref())
}

fn repo_clone_url(provider: WebhookProvider, payload: &WebhookPayload, repo_full_name: &str) -> String {
    payload.repository.as_ref()
        .and_then(|r| r.clone_url.as_deref().or(r.git_http_url.as_deref()))
        .or_else(|| payload.project.as_ref().and_then(|p| p.git_http_url.as_deref().or(p.http_url_to_repo.as_deref())))
        .map(str::to_string)
        .unwrap_or_else(|| match provider {
            WebhookProvider::GitHub => format!("https://github.com/{}.git", repo_full_name),
            WebhookProvider::Gitee => format!("https://gitee.com/{}.git", repo_full_name),
            WebhookProvider::GitLab => format!("https://gitlab.com/{}.git", repo_full_name),
        })
}

fn is_github_ip(addr: SocketAddr, github_ips: &[IpNetwork]) -> bool {
    let ip = addr.ip();
    github_ips.iter().any(|net| net.contains(ip))
}

fn is_allowed_owner(owner_login: &str, owner_type: Option<&str>, allowed_orgs: &[String], allowed_users: &[String]) -> bool {
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

/// PLACEHOLDER_WEBHOOK_HANDLER

pub async fn webhook_handler(
    State(state): State<WebhookState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> StatusCode {
    let provider = match detect_provider(&headers) {
        Some(p) => p,
        None => return StatusCode::BAD_REQUEST,
    };

    let signature = header_value(&headers, "X-Hub-Signature-256");
    let gitee_token = header_value(&headers, "X-Gitee-Token");
    let gitlab_token = header_value(&headers, "X-Gitlab-Token");
    let event_type = match provider {
        WebhookProvider::GitHub => header_value(&headers, "X-GitHub-Event"),
        WebhookProvider::Gitee => header_value(&headers, "X-Gitee-Event"),
        WebhookProvider::GitLab => header_value(&headers, "X-Gitlab-Event"),
    };
    let delivery_id = match provider {
        WebhookProvider::GitHub => header_value(&headers, "X-GitHub-Delivery"),
        WebhookProvider::Gitee => header_value(&headers, "X-Gitee-Timestamp"),
        WebhookProvider::GitLab => header_value(&headers, "X-Gitlab-Event-UUID"),
    };
    let delivery_id = if delivery_id.is_empty() { "unknown" } else { delivery_id };
    let client_addr = get_client_addr(&headers, addr);

    info!(
        "Received {} webhook event: {}, delivery: {}, from: {}",
        provider.as_str(), event_type, delivery_id, client_addr
    );

    if provider == WebhookProvider::GitHub
        && !state.config.skip_ip_check
        && !is_github_ip(client_addr, &state.github_ips)
    {
        return StatusCode::FORBIDDEN;
    }

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

    if !is_push_event(provider, event_type) {
        return StatusCode::OK;
    }

    if !state.config.webhook_secret.is_empty() {
        let verified = match provider {
            WebhookProvider::GitHub => verify_signature(&body, signature, &state.config.webhook_secret),
            WebhookProvider::Gitee => verify_token(gitee_token, &state.config.webhook_secret),
            WebhookProvider::GitLab => verify_token(gitlab_token, &state.config.webhook_secret),
        };
        if !verified {
            return StatusCode::FORBIDDEN;
        }
    }

    let payload: WebhookPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to parse payload: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    let branch = payload.ref_name.as_deref().unwrap_or("unknown").to_string();
    let repo_full_name = payload_repo_full_name(&payload);
    let repo_owner_login = payload_owner_login(&payload);
    let repo_owner_type = payload_owner_type(&payload);
    let committer = payload.head_commit.as_ref()
        .and_then(|c| c.committer.as_ref())
        .and_then(|c| c.name.as_deref())
        .or(payload.user_name.as_deref())
        .unwrap_or("unknown")
        .to_string();
    let repo_url = repo_clone_url(provider, &payload, &repo_full_name);

    if !is_allowed_owner(&repo_owner_login, repo_owner_type, &state.config.allowed_orgs, &state.config.allowed_users) {
        return StatusCode::FORBIDDEN;
    }

    if !["refs/heads/main", "refs/heads/master"].contains(&branch.as_str()) {
        return StatusCode::OK;
    }

    let config = state.config.clone();
    let log_clone = state.deployment_log.clone();

    tokio::spawn(async move {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let repo_name = repo_full_name.split('/').last().unwrap_or(&repo_full_name);

        let venv_settings = VenvSettings {
            use_venv: config.use_venv,
            venv_dir: &config.venv_dir,
            python_bin: &config.python_bin,
            upgrade_pip: config.upgrade_pip,
        };

        let (status, message) = match deploy::run_deployment(
            &config.source_dir, &config.deploy_dir, &repo_url, repo_name, &branch,
            &venv_settings, config.ssh_deploy.as_ref(),
        ).await {
            Ok(output) => ("success".to_string(), output),
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
        let len = log.len();
        if len > 100 {
            let _ = log.split_off(len - 100);
        }
    });

    StatusCode::OK
}

pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "git-webhook-server"
    }))
}

pub async fn get_logs(State(state): State<WebhookState>) -> Json<Vec<DeploymentLog>> {
    let logs = state.deployment_log.read().await;
    Json(logs.clone())
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
    fn gitlab_push_event_is_supported() {
        assert!(is_push_event(WebhookProvider::GitLab, "Push Hook"));
        assert!(is_push_event(WebhookProvider::GitLab, "push"));
        assert!(!is_push_event(WebhookProvider::GitLab, "Merge Request Hook"));
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
        assert!(is_allowed_owner("docs-team", Some("Group"), &allowed_orgs, &allowed_users));
        assert!(!is_allowed_owner("other-team", Some("Group"), &allowed_orgs, &allowed_users));
    }

    #[test]
    fn whitelist_checks_unknown_owner_type_against_both_lists() {
        let allowed_orgs = vec!["docs-team".to_string()];
        let allowed_users = vec!["alice".to_string()];
        assert!(is_allowed_owner("docs-team", None, &allowed_orgs, &allowed_users));
        assert!(is_allowed_owner("alice", None, &allowed_orgs, &allowed_users));
        assert!(!is_allowed_owner("bob", None, &allowed_orgs, &allowed_users));
    }
}
