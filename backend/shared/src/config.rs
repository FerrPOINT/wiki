use config::{Config, ConfigError, Environment, File};
use http::{HeaderValue, Uri};
use serde::{Deserialize, Serialize};
use std::{env, path::Path};

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub environment: RuntimeEnvironment,
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub auth: AuthConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub maintenance: MaintenanceConfig,
    #[serde(default)]
    pub email: EmailConfig,
    #[serde(default)]
    pub bootstrap: BootstrapConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeEnvironment {
    #[default]
    Development,
    Test,
    Staging,
    Production,
}

impl RuntimeEnvironment {
    pub fn is_production(self) -> bool {
        matches!(self, Self::Production)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BootstrapConfig {
    pub admin_email: Option<String>,
    pub admin_username: Option<String>,
    pub admin_password: Option<String>,
    pub admin_display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_address: String,
    pub from_name: String,
    pub starttls: bool,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: String::new(),
            port: 587,
            username: None,
            password: None,
            from_address: String::new(),
            from_name: "Wiki".to_string(),
            starttls: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub dir: String,
    pub max_upload_bytes: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            dir: "/var/lib/wiki/uploads".to_string(),
            max_upload_bytes: 25 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub staged_attachment_ttl_hours: u32,
    pub batch_size: u32,
}

impl Default for MaintenanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 3600,
            staged_attachment_ttl_hours: 24,
            batch_size: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub cors_allowed_origins: Vec<String>,
    /// Auth endpoints rate limit: burst size (requests per period per IP).
    #[serde(default = "default_auth_rate_burst")]
    pub auth_rate_burst: u32,
    /// Auth endpoints rate limit period in seconds.
    #[serde(default = "default_auth_rate_period_secs")]
    pub auth_rate_period_secs: u64,
    /// General API rate limit: burst size.
    #[serde(default = "default_general_rate_burst")]
    pub general_rate_burst: u32,
    /// General API rate limit period in seconds.
    #[serde(default = "default_general_rate_period_secs")]
    pub general_rate_period_secs: u64,
}

fn default_auth_rate_burst() -> u32 {
    5
}
fn default_auth_rate_period_secs() -> u64 {
    15
}
fn default_general_rate_burst() -> u32 {
    60
}
fn default_general_rate_period_secs() -> u64 {
    60
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_token_ttl_minutes: u64,
    pub refresh_token_ttl_days: u64,
    pub registration_enabled: bool,
    pub refresh_cookie_name: String,
    pub refresh_cookie_secure: bool,
    pub refresh_cookie_same_site: String,
    pub refresh_cookie_domain: Option<String>,
    pub refresh_cookie_path: String,
}

impl AppConfig {
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.address, self.server.port)
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_path("config/default.toml")
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let defaults = Config::builder()
            .set_default("environment", "development")?
            .set_default("database.url", "")?
            .set_default("database.max_connections", 20u64)?
            .set_default("database.min_connections", 5u64)?
            .set_default("database.connect_timeout_seconds", 10u64)?
            .set_default("database.idle_timeout_seconds", 600u64)?
            .set_default("server.address", "0.0.0.0")?
            .set_default("server.port", 3456u16)?
            .set_default("server.cors_allowed_origins", vec!["*"])?
            .set_default("server.auth_rate_burst", 5u32)?
            .set_default("server.auth_rate_period_secs", 15u64)?
            .set_default("server.general_rate_burst", 60u32)?
            .set_default("server.general_rate_period_secs", 60u64)?
            .set_default("auth.jwt_secret", "[CHANGE_ME]")?
            .set_default("auth.access_token_ttl_minutes", 15u64)?
            .set_default("auth.refresh_token_ttl_days", 7u64)?
            .set_default("auth.registration_enabled", true)?
            .set_default("auth.refresh_cookie_name", "refresh_token")?
            .set_default("auth.refresh_cookie_secure", true)?
            .set_default("auth.refresh_cookie_same_site", "Lax")?
            .set_default("auth.refresh_cookie_domain", Option::<String>::None)?
            .set_default("auth.refresh_cookie_path", "/api/v1/auth")?
            .set_default("storage.dir", "/var/lib/wiki/uploads")?
            .set_default("storage.max_upload_bytes", 26214400u64)?
            .set_default("maintenance.enabled", true)?
            .set_default("maintenance.interval_seconds", 3600u64)?
            .set_default("maintenance.staged_attachment_ttl_hours", 24u32)?
            .set_default("maintenance.batch_size", 100u32)?
            .set_default("bootstrap.admin_email", Option::<String>::None)?
            .set_default("bootstrap.admin_username", Option::<String>::None)?
            .set_default("bootstrap.admin_password", Option::<String>::None)?
            .set_default("bootstrap.admin_display_name", Option::<String>::None)?
            .set_default("email.enabled", false)?
            .set_default("email.host", "")?
            .set_default("email.port", 587u16)?
            .set_default("email.username", Option::<String>::None)?
            .set_default("email.password", Option::<String>::None)?
            .set_default("email.from_address", "")?
            .set_default("email.from_name", "Wiki")?
            .set_default("email.starttls", true)?
            .build()?;

        let mut cfg: AppConfig = Config::builder()
            .add_source(defaults)
            .add_source(File::from(path.as_ref()).required(false))
            .add_source(
                Environment::with_prefix("WIKI")
                    .separator("__")
                    .prefix_separator("_")
                    .list_separator(",")
                    .with_list_parse_key("server.cors_allowed_origins")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize()?;

        // Backwards-compatible alias: WIKI_JWT_SECRET maps to auth.jwt_secret
        if let Ok(secret) = env::var("WIKI_JWT_SECRET") {
            cfg.auth.jwt_secret = secret;
        }
        if let Ok(email) = env::var("WIKI_ADMIN_EMAIL") {
            cfg.bootstrap.admin_email = Some(email);
        }
        if let Ok(username) = env::var("WIKI_ADMIN_USERNAME") {
            cfg.bootstrap.admin_username = Some(username);
        }
        if let Ok(password) = env::var("WIKI_ADMIN_PASSWORD") {
            cfg.bootstrap.admin_password = Some(password);
        }
        if let Ok(display_name) = env::var("WIKI_ADMIN_DISPLAY_NAME") {
            cfg.bootstrap.admin_display_name = Some(display_name);
        }

        if cfg.auth.jwt_secret == "[CHANGE_ME]" {
            return Err(ConfigError::Message(
                "auth.jwt_secret must be changed from default [CHANGE_ME]".to_string(),
            ));
        }

        cfg.server.cors_allowed_origins =
            normalize_cors_allowed_origins(&cfg.server.cors_allowed_origins, cfg.environment)?;

        if cfg.environment.is_production() {
            validate_production_config(&cfg)?;
        }

        let has_bootstrap_email = cfg
            .bootstrap
            .admin_email
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());
        let has_bootstrap_password = cfg
            .bootstrap
            .admin_password
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());
        if has_bootstrap_email != has_bootstrap_password {
            return Err(ConfigError::Message(
                "bootstrap admin email and password must be set together".to_string(),
            ));
        }

        if cfg.email.enabled {
            if cfg.email.host.trim().is_empty() {
                return Err(ConfigError::Message(
                    "email.host must be set when email is enabled".to_string(),
                ));
            }
            if !is_valid_mail_address(&cfg.email.from_address) {
                return Err(ConfigError::Message(
                    "email.from_address must be a valid mail address".to_string(),
                ));
            }
            let has_user = cfg.email.username.is_some();
            let has_pass = cfg.email.password.is_some();
            if has_user != has_pass {
                return Err(ConfigError::Message(
                    "email.username and email.password must be set together or both omitted"
                        .to_string(),
                ));
            }
        }

        // Rate limits: governor panics on a zero period; validate early so a
        // bad config surfaces as a configuration error, not a startup crash.
        if cfg.server.auth_rate_period_secs == 0 || cfg.server.general_rate_period_secs == 0 {
            return Err(ConfigError::Message(
                "server rate-limit periods must be greater than zero".to_string(),
            ));
        }
        if cfg.server.auth_rate_burst == 0 || cfg.server.general_rate_burst == 0 {
            return Err(ConfigError::Message(
                "server rate-limit bursts must be at least 1".to_string(),
            ));
        }
        if cfg.maintenance.interval_seconds == 0 {
            return Err(ConfigError::Message(
                "maintenance.interval_seconds must be greater than zero".to_string(),
            ));
        }
        if cfg.maintenance.staged_attachment_ttl_hours == 0 {
            return Err(ConfigError::Message(
                "maintenance.staged_attachment_ttl_hours must be greater than zero".to_string(),
            ));
        }
        if cfg.maintenance.batch_size == 0 || cfg.maintenance.batch_size > 1000 {
            return Err(ConfigError::Message(
                "maintenance.batch_size must be between 1 and 1000".to_string(),
            ));
        }

        Ok(cfg)
    }
}

/// Minimal RFC-ish local@domain sanity check for the from-address.
fn is_valid_mail_address(addr: &str) -> bool {
    let addr = addr.trim();
    if addr.is_empty() || !addr.contains('@') {
        return false;
    }
    let (local, domain) = addr.split_once('@').unwrap();
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

fn normalize_cors_allowed_origins(
    origins: &[String],
    environment: RuntimeEnvironment,
) -> Result<Vec<String>, ConfigError> {
    let mut normalized = Vec::new();
    for raw_origin in origins {
        let origin = raw_origin.trim();
        if origin.is_empty() {
            continue;
        }
        validate_cors_origin(origin, environment)?;
        if !normalized.iter().any(|existing| existing == origin) {
            normalized.push(origin.to_string());
        }
    }
    if normalized.is_empty() {
        return Err(ConfigError::Message(
            "server.cors_allowed_origins must contain at least one origin".to_string(),
        ));
    }
    Ok(normalized)
}

fn validate_cors_origin(origin: &str, environment: RuntimeEnvironment) -> Result<(), ConfigError> {
    if origin == "*" {
        if environment.is_production() {
            return Err(ConfigError::Message(
                "server.cors_allowed_origins must not include wildcard '*' when environment=production"
                    .to_string(),
            ));
        }
        return Ok(());
    }
    if origin.ends_with('/') {
        return Err(ConfigError::Message(format!(
            "server.cors_allowed_origins must not include path or query: {origin}"
        )));
    }

    HeaderValue::from_str(origin).map_err(|_| {
        ConfigError::Message(format!(
            "server.cors_allowed_origins contains invalid header value: {origin}"
        ))
    })?;
    let uri = origin.parse::<Uri>().map_err(|_| {
        ConfigError::Message(format!(
            "server.cors_allowed_origins must contain valid origins: {origin}"
        ))
    })?;
    let scheme = uri.scheme_str().ok_or_else(|| {
        ConfigError::Message(format!(
            "server.cors_allowed_origins origin must include http or https scheme: {origin}"
        ))
    })?;
    if scheme != "http" && scheme != "https" {
        return Err(ConfigError::Message(format!(
            "server.cors_allowed_origins origin must use http or https scheme: {origin}"
        )));
    }
    if environment.is_production() && scheme != "https" {
        return Err(ConfigError::Message(format!(
            "server.cors_allowed_origins production origins must use https: {origin}"
        )));
    }
    if uri.authority().is_none() {
        return Err(ConfigError::Message(format!(
            "server.cors_allowed_origins origin must include host: {origin}"
        )));
    }
    if uri
        .path_and_query()
        .is_some_and(|path_and_query| path_and_query.as_str() != "/")
    {
        return Err(ConfigError::Message(format!(
            "server.cors_allowed_origins must not include path or query: {origin}"
        )));
    }
    Ok(())
}

fn validate_production_config(cfg: &AppConfig) -> Result<(), ConfigError> {
    if cfg.database.url.trim().is_empty() {
        return Err(ConfigError::Message(
            "database.url must be set when environment=production".to_string(),
        ));
    }
    if cfg.auth.jwt_secret.trim().len() < 32 {
        return Err(ConfigError::Message(
            "auth.jwt_secret must be at least 32 characters when environment=production"
                .to_string(),
        ));
    }
    if !cfg.auth.refresh_cookie_secure {
        return Err(ConfigError::Message(
            "auth.refresh_cookie_secure must be true when environment=production".to_string(),
        ));
    }
    Ok(())
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 20,
            min_connections: 5,
            connect_timeout_seconds: 10,
            idle_timeout_seconds: 600,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0".to_string(),
            port: 3456,
            cors_allowed_origins: vec!["*".to_string()],
            auth_rate_burst: default_auth_rate_burst(),
            auth_rate_period_secs: default_auth_rate_period_secs(),
            general_rate_burst: default_general_rate_burst(),
            general_rate_period_secs: default_general_rate_period_secs(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "[CHANGE_ME]".to_string(),
            access_token_ttl_minutes: 15,
            refresh_token_ttl_days: 7,
            registration_enabled: true,
            refresh_cookie_name: "refresh_token".to_string(),
            refresh_cookie_secure: true,
            refresh_cookie_same_site: "Lax".to_string(),
            refresh_cookie_domain: None,
            refresh_cookie_path: "/api/v1/auth".to_string(),
        }
    }
}
