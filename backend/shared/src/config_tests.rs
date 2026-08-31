use std::env;
use std::sync::Mutex;

use crate::AppConfig;

// Tests that mutate process-wide env vars must not run in parallel.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn clear_env() {
    for key in [
        "WIKI_DATABASE__URL",
        "WIKI_DATABASE__MAX_CONNECTIONS",
        "WIKI_DATABASE__MIN_CONNECTIONS",
        "WIKI_DATABASE__CONNECT_TIMEOUT_SECONDS",
        "WIKI_DATABASE__IDLE_TIMEOUT_SECONDS",
        "WIKI_SERVER__ADDRESS",
        "WIKI_SERVER__PORT",
        "WIKI_AUTH__JWT_SECRET",
        "WIKI_JWT_SECRET",
        "WIKI_AUTH__ACCESS_TOKEN_TTL_MINUTES",
        "WIKI_AUTH__REFRESH_TOKEN_TTL_DAYS",
        "WIKI_AUTH__REFRESH_TOKEN_COOKIE_NAME",
        "WIKI_AUTH__REFRESH_COOKIE_SECURE",
        "WIKI_AUTH__REFRESH_COOKIE_SAME_SITE",
        "WIKI_AUTH__REFRESH_COOKIE_DOMAIN",
        "WIKI_AUTH__REFRESH_COOKIE_PATH",
        "WIKI_EMAIL__ENABLED",
        "WIKI_EMAIL__HOST",
        "WIKI_EMAIL__PORT",
        "WIKI_EMAIL__USERNAME",
        "WIKI_EMAIL__PASSWORD",
        "WIKI_EMAIL__FROM_ADDRESS",
        "WIKI_EMAIL__FROM_NAME",
        "WIKI_EMAIL__STARTTLS",
        "WIKI_SERVER__AUTH_RATE_BURST",
        "WIKI_SERVER__AUTH_RATE_PERIOD_SECS",
        "WIKI_SERVER__GENERAL_RATE_BURST",
        "WIKI_SERVER__GENERAL_RATE_PERIOD_SECS",
        "WIKI_BOOTSTRAP__ADMIN_EMAIL",
        "WIKI_BOOTSTRAP__ADMIN_USERNAME",
        "WIKI_BOOTSTRAP__ADMIN_PASSWORD",
        "WIKI_BOOTSTRAP__ADMIN_DISPLAY_NAME",
        "WIKI_ADMIN_EMAIL",
        "WIKI_ADMIN_USERNAME",
        "WIKI_ADMIN_PASSWORD",
        "WIKI_ADMIN_DISPLAY_NAME",
    ] {
        unsafe { env::remove_var(key) };
    }
}

fn set_env(key: &str, value: &str) {
    unsafe { env::set_var(key, value) };
}

#[test]
fn config_scenarios() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_env();
    set_env("WIKI_JWT_SECRET", "test-secret-32-chars-long!!!!!");

    // Defaults
    let cfg = AppConfig::from_path("/nonexistent.toml").unwrap();
    assert_eq!(cfg.server.address, "0.0.0.0");
    assert_eq!(cfg.server.port, 3456);
    assert_eq!(cfg.server_addr(), "0.0.0.0:3456");
    assert_eq!(cfg.database.max_connections, 20);
    assert_eq!(cfg.database.min_connections, 5);
    assert_eq!(cfg.database.connect_timeout_seconds, 10);
    assert_eq!(cfg.database.idle_timeout_seconds, 600);
    assert_eq!(cfg.auth.access_token_ttl_minutes, 15);
    assert_eq!(cfg.auth.refresh_token_ttl_days, 7);
    assert_eq!(cfg.auth.refresh_cookie_name, "refresh_token");
    assert!(cfg.auth.refresh_cookie_secure);
    assert_eq!(cfg.auth.refresh_cookie_same_site, "Lax");
    assert_eq!(cfg.auth.refresh_cookie_path, "/api/v1/auth");
    assert_eq!(cfg.database.url, "");
    assert_eq!(cfg.auth.jwt_secret, "test-secret-32-chars-long!!!!!");
    assert_eq!(cfg.bootstrap.admin_email, None);
    set_env("WIKI_DATABASE__URL", "postgres://u:***@localhost:5432/db");
    set_env("WIKI_DATABASE__MAX_CONNECTIONS", "42");
    set_env("WIKI_DATABASE__MIN_CONNECTIONS", "3");
    set_env("WIKI_DATABASE__CONNECT_TIMEOUT_SECONDS", "5");
    set_env("WIKI_DATABASE__IDLE_TIMEOUT_SECONDS", "300");
    set_env("WIKI_SERVER__PORT", "19876");
    set_env("WIKI_AUTH__ACCESS_TOKEN_TTL_MINUTES", "60");
    set_env("WIKI_AUTH__REFRESH_TOKEN_TTL_DAYS", "14");
    set_env("WIKI_AUTH__JWT_SECRET", "test-secret-32-chars-long!!!!!");
    let cfg = AppConfig::from_path("/nonexistent.toml").unwrap();
    assert_eq!(cfg.server.port, 19876);
    assert_eq!(cfg.auth.jwt_secret, "test-secret-32-chars-long!!!!!");
    assert!(cfg.database.url.contains("localhost:5432/db"));

    // Environment separator is `__`, so nested keys become
    // `WIKI_DATABASE__CONNECT_TIMEOUT_SECONDS`.
    assert_eq!(cfg.database.connect_timeout_seconds, 5);
    assert_eq!(cfg.database.idle_timeout_seconds, 300);
    assert_eq!(cfg.database.max_connections, 42);
    assert_eq!(cfg.database.min_connections, 3);

    set_env("WIKI_ADMIN_EMAIL", "admin@example.test");
    set_env("WIKI_ADMIN_USERNAME", "admin");
    set_env("WIKI_ADMIN_PASSWORD", "admin-secret");
    set_env("WIKI_ADMIN_DISPLAY_NAME", "Wiki Admin");
    let cfg = AppConfig::from_path("/nonexistent.toml").unwrap();
    assert_eq!(
        cfg.bootstrap.admin_email.as_deref(),
        Some("admin@example.test")
    );
    assert_eq!(cfg.bootstrap.admin_username.as_deref(), Some("admin"));
    assert_eq!(
        cfg.bootstrap.admin_password.as_deref(),
        Some("admin-secret")
    );
    assert_eq!(
        cfg.bootstrap.admin_display_name.as_deref(),
        Some("Wiki Admin")
    );

    set_env("WIKI_SERVER__PORT", "not-a-number");
    let err = AppConfig::from_path("/nonexistent.toml").unwrap_err();
    assert!(err.to_string().contains("invalid type"));

    clear_env();
}

#[test]
fn config_defaults_implemented() {
    let _guard = ENV_LOCK.lock().unwrap();
    let cfg = AppConfig::default();
    assert_eq!(cfg.server.port, 3456);
    assert_eq!(cfg.database.max_connections, 20);
    assert_eq!(cfg.auth.jwt_secret, "[CHANGE_ME]");
}

#[test]
fn email_defaults_are_disabled_and_safe() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_env();
    set_env("WIKI_JWT_SECRET", "test-secret-32-chars-long!!!!!");

    let cfg = AppConfig::from_path("/nonexistent.toml").unwrap();

    assert!(!cfg.email.enabled);
    assert_eq!(cfg.email.host, "");
    assert_eq!(cfg.email.port, 587);
    assert_eq!(cfg.email.username, None);
    assert_eq!(cfg.email.password, None);
    assert_eq!(cfg.email.from_address, "");
    assert_eq!(cfg.email.from_name, "Wiki");
    assert!(cfg.email.starttls);

    clear_env();
}

#[test]
fn enabled_email_requires_a_complete_valid_configuration() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_env();
    set_env("WIKI_JWT_SECRET", "test-secret-32-chars-long!!!!!");
    set_env("WIKI_EMAIL__ENABLED", "true");
    set_env("WIKI_EMAIL__FROM_ADDRESS", "noreply@example.test");

    let err = AppConfig::from_path("/nonexistent.toml").unwrap_err();
    assert!(err.to_string().contains("email.host"));

    set_env("WIKI_EMAIL__HOST", "smtp.example.test");
    set_env("WIKI_EMAIL__FROM_ADDRESS", "not an email");
    let err = AppConfig::from_path("/nonexistent.toml").unwrap_err();
    assert!(err.to_string().contains("email.from_address"));

    set_env("WIKI_EMAIL__FROM_ADDRESS", "noreply@example.test");
    set_env("WIKI_EMAIL__USERNAME", "mailer");
    let err = AppConfig::from_path("/nonexistent.toml").unwrap_err();
    assert!(
        err.to_string()
            .contains("email.username and email.password")
    );

    set_env("WIKI_EMAIL__PASSWORD", "test-password");
    let cfg = AppConfig::from_path("/nonexistent.toml").unwrap();
    assert!(cfg.email.enabled);
    assert_eq!(cfg.email.host, "smtp.example.test");
    assert_eq!(cfg.email.username.as_deref(), Some("mailer"));
    assert_eq!(cfg.email.password.as_deref(), Some("test-password"));

    clear_env();
}

#[test]
fn config_from_env_uses_default_path() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_env();
    set_env("WIKI_JWT_SECRET", "test-secret-32-chars-long!!!!!");
    // from_env targets config/default.toml which won't exist; defaults still apply
    let cfg = AppConfig::from_env().unwrap();
    assert_eq!(cfg.server.port, 3456);
    clear_env();
}

#[test]
fn rate_limit_defaults_and_env_override() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_env();
    set_env("WIKI_JWT_SECRET", "test-secret-32-chars-long!!!!!");

    let cfg = AppConfig::from_path("/nonexistent.toml").unwrap();
    assert_eq!(cfg.server.auth_rate_burst, 5);
    assert_eq!(cfg.server.auth_rate_period_secs, 15);
    assert_eq!(cfg.server.general_rate_burst, 60);
    assert_eq!(cfg.server.general_rate_period_secs, 60);

    set_env("WIKI_SERVER__AUTH_RATE_BURST", "100");
    set_env("WIKI_SERVER__AUTH_RATE_PERIOD_SECS", "1");
    set_env("WIKI_SERVER__GENERAL_RATE_BURST", "10000");
    set_env("WIKI_SERVER__GENERAL_RATE_PERIOD_SECS", "1");
    set_env("WIKI_AUTH__JWT_SECRET", "test-secret-123");

    let cfg = AppConfig::from_path("/nonexistent.toml").unwrap();
    assert_eq!(cfg.server.auth_rate_burst, 100);
    assert_eq!(cfg.server.auth_rate_period_secs, 1);
    assert_eq!(cfg.server.general_rate_burst, 10000);
    assert_eq!(cfg.server.general_rate_period_secs, 1);
}

#[test]
fn rate_limit_zero_values_rejected() {
    let _guard = ENV_LOCK.lock().unwrap();

    clear_env();
    unsafe { env::set_var("WIKI_SERVER__AUTH_RATE_BURST", "0") };
    let err = AppConfig::from_path("/nonexistent.toml");
    assert!(err.is_err(), "zero auth burst must be a config error");

    clear_env();
    unsafe { env::set_var("WIKI_SERVER__GENERAL_RATE_PERIOD_SECS", "0") };
    let err = AppConfig::from_path("/nonexistent.toml");
    assert!(err.is_err(), "zero general period must be a config error");
}

#[test]
fn bootstrap_admin_requires_email_and_password_together() {
    let _guard = ENV_LOCK.lock().unwrap();

    clear_env();
    set_env("WIKI_JWT_SECRET", "test-secret-32-chars-long!!!!!");
    set_env("WIKI_ADMIN_EMAIL", "admin@example.test");
    let err = AppConfig::from_path("/nonexistent.toml");
    assert!(err.is_err(), "email without password must be rejected");

    clear_env();
    set_env("WIKI_JWT_SECRET", "test-secret-32-chars-long!!!!!");
    set_env("WIKI_ADMIN_PASSWORD", "admin-secret");
    let err = AppConfig::from_path("/nonexistent.toml");
    assert!(err.is_err(), "password without email must be rejected");

    clear_env();
}
