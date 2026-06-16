use serde::Deserialize;
use std::env;
use std::path::Path;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ConfigFile {
    pub server: Option<ServerConfig>,
    pub admin: Option<AdminConfig>,
    pub user: Option<UserConfig>,
    pub webhook: Option<WebhookConfig>,
    pub build: Option<BuildConfig>,
    pub deploy_ssh: Option<SshDeployConfig>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ServerConfig {
    pub port: Option<u16>,
    pub log_file_path: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AdminConfig {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UserConfig {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WebhookConfig {
    pub secret: Option<String>,
    pub allowed_orgs: Option<Vec<String>>,
    pub allowed_users: Option<Vec<String>>,
    pub skip_ip_check: Option<bool>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BuildConfig {
    pub source_dir: Option<String>,
    pub deploy_dir: Option<String>,
    pub use_venv: Option<bool>,
    pub venv_dir: Option<String>,
    pub python_bin: Option<String>,
    pub upgrade_pip: Option<bool>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SshDeployConfig {
    pub enabled: Option<bool>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub remote_path: Option<String>,
}

/// Resolved application configuration (merged from config.toml + env vars).
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub log_file_path: String,
    pub webhook_secret: String,
    pub allowed_orgs: Vec<String>,
    pub allowed_users: Vec<String>,
    pub skip_ip_check: bool,
    pub source_dir: String,
    pub deploy_dir: String,
    pub use_venv: bool,
    pub venv_dir: String,
    pub python_bin: String,
    pub upgrade_pip: bool,
    pub admin: Option<ResolvedAdminConfig>,
    pub user: Option<ResolvedUserConfig>,
    pub ssh_deploy: Option<ResolvedSshConfig>,
}

#[derive(Clone, Debug)]
pub struct ResolvedAdminConfig {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct ResolvedUserConfig {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct ResolvedSshConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub remote_path: String,
}

impl AppConfig {
    pub fn load() -> Self {
        let file_config = load_config_file();

        let server = file_config.server.unwrap_or(ServerConfig {
            port: None,
            log_file_path: None,
        });
        let webhook = file_config.webhook.unwrap_or(WebhookConfig {
            secret: None,
            allowed_orgs: None,
            allowed_users: None,
            skip_ip_check: None,
        });
        let build = file_config.build.unwrap_or(BuildConfig {
            source_dir: None,
            deploy_dir: None,
            use_venv: None,
            venv_dir: None,
            python_bin: None,
            upgrade_pip: None,
        });

        let port = env_or("PORT", server.port.map(|p| p.to_string()))
            .and_then(|s| s.parse().ok())
            .unwrap_or(5000);

        let log_file_path = env_or("LOG_FILE_PATH", server.log_file_path)
            .unwrap_or_else(|| "webhook-server.log".to_string());

        let webhook_secret =
            env_or("WEBHOOK_SECRET", webhook.secret).unwrap_or_default();

        let allowed_orgs = env_list_or("ALLOWED_ORGS", webhook.allowed_orgs);
        let allowed_users = env_list_or("ALLOWED_USERS", webhook.allowed_users);

        let skip_ip_check = env_bool_or("SKIP_IP_CHECK", webhook.skip_ip_check);

        let source_dir = env_or("SOURCE_DIR", build.source_dir)
            .unwrap_or_else(|| "/var/www/docs-source".to_string());
        let deploy_dir = env_or("DEPLOY_DIR", build.deploy_dir)
            .unwrap_or_else(|| "/var/www/docs".to_string());
        let use_venv = env_bool_or("USE_VENV", build.use_venv.or(Some(true)));
        let venv_dir =
            env_or("VENV_DIR", build.venv_dir).unwrap_or_else(|| "venv".to_string());
        let python_bin = env_or("PYTHON_BIN", build.python_bin)
            .unwrap_or_else(|| "python3".to_string());
        let upgrade_pip = env_bool_or("UPGRADE_PIP", build.upgrade_pip.or(Some(true)));

        let admin = file_config.admin.and_then(|a| {
            let username = a.username.unwrap_or_else(|| "admin".to_string());
            let password = a.password?;
            Some(ResolvedAdminConfig { username, password })
        });

        let user = file_config.user.and_then(|u| {
            let username = u.username.unwrap_or_else(|| "user".to_string());
            let password = u.password?;
            Some(ResolvedUserConfig { username, password })
        });

        let ssh_deploy = file_config.deploy_ssh.and_then(|s| {
            if !s.enabled.unwrap_or(false) {
                return None;
            }
            Some(ResolvedSshConfig {
                host: s.host?,
                port: s.port.unwrap_or(22),
                user: s.user.unwrap_or_else(|| "root".to_string()),
                password: s.password?,
                remote_path: s.remote_path?,
            })
        });

        Self {
            port,
            log_file_path,
            webhook_secret,
            allowed_orgs,
            allowed_users,
            skip_ip_check,
            source_dir,
            deploy_dir,
            use_venv,
            venv_dir,
            python_bin,
            upgrade_pip,
            admin,
            user,
            ssh_deploy,
        }
    }
}

fn load_config_file() -> ConfigFile {
    let path = env::var("CONFIG_FILE").unwrap_or_else(|_| "config.toml".to_string());
    if !Path::new(&path).exists() {
        return ConfigFile::default();
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    toml::from_str(&content).unwrap_or_default()
}

fn env_or(key: &str, fallback: Option<String>) -> Option<String> {
    env::var(key).ok().or(fallback)
}

fn env_list_or(key: &str, fallback: Option<Vec<String>>) -> Vec<String> {
    if let Ok(val) = env::var(key) {
        return val
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    fallback.unwrap_or_default()
}

fn env_bool_or(key: &str, fallback: Option<bool>) -> bool {
    if let Ok(val) = env::var(key) {
        return val.parse().unwrap_or(false);
    }
    fallback.unwrap_or(false)
}
