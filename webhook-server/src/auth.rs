use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::{ResolvedAdminConfig, ResolvedUserConfig};

#[derive(Clone)]
pub struct AuthState {
    admin: ResolvedAdminConfig,
    user: Option<ResolvedUserConfig>,
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
}

#[derive(Clone)]
struct SessionInfo {
    role: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub role: String,
}

impl AuthState {
    pub fn new(admin: ResolvedAdminConfig, user: Option<ResolvedUserConfig>) -> Self {
        Self {
            admin,
            user,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn validate_token_role(&self, token: &str) -> Option<String> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(token) {
            let elapsed = chrono::Utc::now() - session.created_at;
            if elapsed.num_hours() < 24 {
                return Some(session.role.clone());
            }
        }
        None
    }
}

pub async fn login(
    State(auth): State<AuthState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let role = if req.username == auth.admin.username && req.password == auth.admin.password {
        "admin".to_string()
    } else if let Some(ref user) = auth.user {
        if req.username == user.username && req.password == user.password {
            "user".to_string()
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let token = generate_token();
    let session = SessionInfo {
        role: role.clone(),
        created_at: chrono::Utc::now(),
    };

    auth.sessions.write().await.insert(token.clone(), session);

    Ok(Json(LoginResponse { token, role }))
}

pub fn extract_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
}

fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}
