mod admin;
mod auth;
mod config;
mod deploy;
mod webhook;

use axum::{
    routing::{delete, get, post},
    Router,
};
use ipnetwork::IpNetwork;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::AppConfig;
use crate::webhook::WebhookState;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let config = AppConfig::load();

    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        std::env::current_dir().unwrap_or_default(),
        &config.log_file_path,
    );

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

    let port = config.port;
    let config = Arc::new(config);

    let github_ips: Vec<IpNetwork> = webhook::GITHUB_WEBHOOK_IPS
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect();

    let webhook_state = WebhookState {
        config: config.clone(),
        deployment_log: Arc::new(RwLock::new(Vec::new())),
        github_ips: Arc::new(github_ips),
    };

    let mut app = Router::new()
        .route("/webhook", post(webhook::webhook_handler))
        .route("/health", get(webhook::health_check))
        .route("/logs", get(webhook::get_logs))
        .with_state(webhook_state);

    if let Some(ref admin_config) = config.admin {
        let auth_state = auth::AuthState::new(admin_config.clone(), config.user.clone());
        let admin_state = admin::AdminState::new(auth_state, config.clone());

        let admin_router = Router::new()
            .route("/admin", get(admin::admin_page))
            .route("/admin/login", post(admin::login))
            .route("/admin/me", get(admin::me))
            .route("/admin/repos", get(admin::list_repos))
            .route("/admin/repos", post(admin::add_repo))
            .route("/admin/repos/:id", delete(admin::delete_repo))
            .route("/admin/repos/:id/sync", post(admin::sync_repo))
            .route("/admin/repos/:id/build", post(admin::build_repo))
            .route("/admin/repos/:id/deploy", post(admin::deploy_repo))
            .with_state(admin_state);

        app = app.merge(admin_router);
        info!("Admin panel enabled at /admin");
    }

    info!("Starting webhook server on port {}", port);
    info!("Source directory: {}", config.source_dir);
    info!("Deploy directory: {}", config.deploy_dir);
    info!(
        "SSH deploy: {}",
        if config.ssh_deploy.is_some() { "enabled" } else { "disabled" }
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
