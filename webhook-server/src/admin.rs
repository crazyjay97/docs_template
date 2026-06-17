use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Html,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::auth::{self, AuthState};
use crate::config::AppConfig;
use crate::deploy::{self, VenvSettings};

const ADMIN_HTML: &str = include_str!("web/index.html");

#[derive(Clone)]
pub struct AdminState {
    pub auth: AuthState,
    pub config: Arc<AppConfig>,
    pub repos: Arc<RwLock<Vec<RepoEntry>>>,
    pub repo_file: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RepoEntry {
    pub id: String,
    pub name: String,
    pub git_url: String,
    pub branch: String,
    #[serde(default)]
    pub deploy_name: Option<String>,
    pub added_at: String,
    pub last_sync: Option<String>,
    pub last_build: Option<String>,
    pub last_deploy: Option<String>,
    pub status: String,
    pub last_message: Option<String>,
}

#[derive(Deserialize)]
pub struct AddRepoRequest {
    pub name: String,
    pub git_url: String,
    pub branch: Option<String>,
    pub deploy_name: Option<String>,
}

#[derive(Deserialize)]
pub struct EditRepoRequest {
    pub name: Option<String>,
    pub git_url: Option<String>,
    pub branch: Option<String>,
    pub deploy_name: Option<String>,
}

impl AdminState {
    pub fn new(auth: AuthState, config: Arc<AppConfig>) -> Self {
        let repo_file = "repos.json".to_string();
        let repos = load_repos(&repo_file);
        Self {
            auth,
            config,
            repos: Arc::new(RwLock::new(repos)),
            repo_file,
        }
    }

    async fn check_auth(&self, headers: &HeaderMap) -> Result<String, StatusCode> {
        let token = auth::extract_token(headers).ok_or(StatusCode::UNAUTHORIZED)?;
        self.auth
            .validate_token_role(token)
            .await
            .ok_or(StatusCode::UNAUTHORIZED)
    }

    async fn require_admin(&self, headers: &HeaderMap) -> Result<(), StatusCode> {
        let role = self.check_auth(headers).await?;
        if role != "admin" {
            return Err(StatusCode::FORBIDDEN);
        }
        Ok(())
    }

    async fn save(&self) {
        let repos = self.repos.read().await;
        if let Ok(json) = serde_json::to_string_pretty(&*repos) {
            let _ = tokio::fs::write(&self.repo_file, json).await;
        }
    }
}

pub async fn admin_page() -> Html<&'static str> {
    Html(ADMIN_HTML)
}

pub async fn login(State(state): State<AdminState>, body: Json<auth::LoginRequest>) -> Result<Json<auth::LoginResponse>, StatusCode> {
    auth::login(State(state.auth), body).await
}

pub async fn me(
    State(state): State<AdminState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let role = state.check_auth(&headers).await?;
    Ok(Json(serde_json::json!({"role": role})))
}

pub async fn list_repos(
    State(state): State<AdminState>,
    headers: HeaderMap,
) -> Result<Json<Vec<RepoEntry>>, StatusCode> {
    let role = state.check_auth(&headers).await?;
    let repos = state.repos.read().await;
    let mut result = repos.clone();
    if role != "admin" {
        for r in &mut result {
            if let Some(msg) = &r.last_message {
                if msg.contains("SSH deployed") || msg.contains("SSH deploy") {
                    r.last_message = Some("Published successfully".to_string());
                }
            }
        }
    }
    Ok(Json(result))
}

pub async fn add_repo(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Json(req): Json<AddRepoRequest>,
) -> Result<Json<RepoEntry>, StatusCode> {
    state.require_admin(&headers).await?;

    let entry = RepoEntry {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name,
        git_url: req.git_url,
        branch: req.branch.unwrap_or_else(|| "main".to_string()),
        deploy_name: req.deploy_name,
        added_at: chrono::Utc::now().to_rfc3339(),
        last_sync: None,
        last_build: None,
        last_deploy: None,
        status: "idle".to_string(),
        last_message: None,
    };

    let mut repos = state.repos.write().await;
    repos.push(entry.clone());
    drop(repos);
    state.save().await;

    info!("Added repo: {} ({})", entry.name, entry.git_url);
    Ok(Json(entry))
}

pub async fn delete_repo(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.require_admin(&headers).await?;
    let mut repos = state.repos.write().await;
    let before = repos.len();
    repos.retain(|r| r.id != id);
    if repos.len() == before {
        return Err(StatusCode::NOT_FOUND);
    }
    drop(repos);
    state.save().await;
    Ok(StatusCode::OK)
}

/// PLACEHOLDER_ACTIONS

pub async fn edit_repo(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<EditRepoRequest>,
) -> Result<Json<RepoEntry>, StatusCode> {
    state.require_admin(&headers).await?;

    let mut repos = state.repos.write().await;
    let repo = repos.iter_mut().find(|r| r.id == id).ok_or(StatusCode::NOT_FOUND)?;

    if let Some(name) = req.name {
        repo.name = name;
    }
    if let Some(git_url) = req.git_url {
        repo.git_url = git_url;
    }
    if let Some(branch) = req.branch {
        repo.branch = branch;
    }
    if let Some(deploy_name) = req.deploy_name {
        repo.deploy_name = if deploy_name.is_empty() { None } else { Some(deploy_name) };
    }

    let updated = repo.clone();
    drop(repos);
    state.save().await;

    info!("Edited repo: {} ({})", updated.name, updated.id);
    Ok(Json(updated))
}

pub async fn sync_repo(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.check_auth(&headers).await?;

    let repo = {
        let repos = state.repos.read().await;
        repos.iter().find(|r| r.id == id).cloned()
    };
    let repo = repo.ok_or(StatusCode::NOT_FOUND)?;

    update_status(&state, &id, "syncing").await;

    let branch_ref = format!("refs/heads/{}", repo.branch);
    let result = deploy::git_sync(&state.config.source_dir, &repo.git_url, &repo.name, &branch_ref).await;

    match &result {
        Ok(msg) => {
            let mut repos = state.repos.write().await;
            if let Some(r) = repos.iter_mut().find(|r| r.id == id) {
                r.status = "idle".to_string();
                r.last_sync = Some(chrono::Utc::now().to_rfc3339());
                r.last_message = Some(msg.clone());
            }
            drop(repos);
            state.save().await;
        }
        Err(e) => {
            let mut repos = state.repos.write().await;
            if let Some(r) = repos.iter_mut().find(|r| r.id == id) {
                r.status = "error".to_string();
                r.last_message = Some(e.clone());
            }
            drop(repos);
            state.save().await;
        }
    }

    match result {
        Ok(msg) => Ok(Json(serde_json::json!({"status": "ok", "message": msg}))),
        Err(e) => Ok(Json(serde_json::json!({"status": "error", "message": e}))),
    }
}

pub async fn build_repo(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.check_auth(&headers).await?;

    let repo = {
        let repos = state.repos.read().await;
        repos.iter().find(|r| r.id == id).cloned()
    };
    let repo = repo.ok_or(StatusCode::NOT_FOUND)?;

    update_status(&state, &id, "building").await;

    let venv = VenvSettings {
        use_venv: state.config.use_venv,
        venv_dir: &state.config.venv_dir,
        python_bin: &state.config.python_bin,
        upgrade_pip: state.config.upgrade_pip,
    };

    let result = deploy::build_docs(
        &state.config.source_dir, &state.config.deploy_dir, &repo.name, &venv,
    ).await;

    match &result {
        Ok(msg) => {
            let mut repos = state.repos.write().await;
            if let Some(r) = repos.iter_mut().find(|r| r.id == id) {
                r.status = "idle".to_string();
                r.last_build = Some(chrono::Utc::now().to_rfc3339());
                r.last_message = Some(msg.clone());
            }
            drop(repos);
            state.save().await;
        }
        Err(e) => {
            let mut repos = state.repos.write().await;
            if let Some(r) = repos.iter_mut().find(|r| r.id == id) {
                r.status = "error".to_string();
                r.last_message = Some(e.clone());
            }
            drop(repos);
            state.save().await;
        }
    }

    match result {
        Ok(msg) => Ok(Json(serde_json::json!({"status": "ok", "message": msg}))),
        Err(e) => Ok(Json(serde_json::json!({"status": "error", "message": e}))),
    }
}

pub async fn deploy_repo(
    State(state): State<AdminState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.require_admin(&headers).await?;

    let repo = {
        let repos = state.repos.read().await;
        repos.iter().find(|r| r.id == id).cloned()
    };
    let repo = repo.ok_or(StatusCode::NOT_FOUND)?;

    let ssh_config = state.config.ssh_deploy.as_ref().ok_or_else(|| {
        StatusCode::BAD_REQUEST
    })?;

    update_status(&state, &id, "deploying").await;

    let remote_name = repo.deploy_name.as_deref().unwrap_or(&repo.name);
    let result = deploy::ssh_deploy(&state.config.deploy_dir, &repo.name, remote_name, ssh_config).await;

    match &result {
        Ok(msg) => {
            let mut repos = state.repos.write().await;
            if let Some(r) = repos.iter_mut().find(|r| r.id == id) {
                r.status = "idle".to_string();
                r.last_deploy = Some(chrono::Utc::now().to_rfc3339());
                r.last_message = Some(msg.clone());
            }
            drop(repos);
            state.save().await;
        }
        Err(e) => {
            let mut repos = state.repos.write().await;
            if let Some(r) = repos.iter_mut().find(|r| r.id == id) {
                r.status = "error".to_string();
                r.last_message = Some(e.clone());
            }
            drop(repos);
            state.save().await;
        }
    }

    match result {
        Ok(msg) => Ok(Json(serde_json::json!({"status": "ok", "message": msg}))),
        Err(e) => Ok(Json(serde_json::json!({"status": "error", "message": e}))),
    }
}

async fn update_status(state: &AdminState, id: &str, status: &str) {
    let mut repos = state.repos.write().await;
    if let Some(r) = repos.iter_mut().find(|r| r.id == id) {
        r.status = status.to_string();
    }
    drop(repos);
    state.save().await;
}

fn load_repos(path: &str) -> Vec<RepoEntry> {
    let mut repos: Vec<RepoEntry> = std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    for r in &mut repos {
        match r.status.as_str() {
            "syncing" | "building" | "deploying" => {
                r.status = "idle".to_string();
            }
            _ => {}
        }
    }
    repos
}
