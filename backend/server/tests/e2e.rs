use std::sync::Arc;

use serial_test::serial;
use server::run;
use shared::{AppConfig, AuthConfig, EmailConfig, ServerConfig};

fn test_config() -> Arc<AppConfig> {
    Arc::new(AppConfig {
        database: shared::DatabaseConfig::default(),
        server: ServerConfig {
            address: "127.0.0.1".to_string(),
            port: 0,
            cors_allowed_origins: vec!["*".to_string()],
            ..Default::default()
        },
        auth: AuthConfig {
            jwt_secret: "test-secret".to_string(),
            access_token_ttl_minutes: 15,
            refresh_token_ttl_days: 7,
            refresh_cookie_name: "refresh_token".to_string(),
            refresh_cookie_secure: true,
            refresh_cookie_same_site: "Lax".to_string(),
            refresh_cookie_domain: None,
            refresh_cookie_path: "/api/v1/auth".to_string(),
        },
        storage: shared::StorageConfig::default(),
        email: EmailConfig::default(),
        bootstrap: shared::BootstrapConfig::default(),
    })
}

#[tokio::test]
#[serial]
async fn server_starts_and_serves_health() {
    let config = test_config();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let handle = tokio::spawn(run(config, ready_tx, shutdown_rx));

    let addr = tokio::time::timeout(std::time::Duration::from_secs(30), ready_rx)
        .await
        .expect("server did not become ready")
        .expect("ready channel closed");
    let url = format!("http://{}/api/v1/health", addr);

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .send()
        .await
        .expect("health request failed");
    assert_eq!(res.status(), 200);
    assert_eq!(res.text().await.unwrap(), "ok");

    // A healthy server must outlive the 30-second graceful-shutdown drain
    // limit. This caught a regression where the serve future itself was timed
    // out, causing a clean container restart every 30 seconds.
    tokio::time::sleep(std::time::Duration::from_secs(31)).await;
    let res = client
        .get(&url)
        .send()
        .await
        .expect("health request after server lifetime check failed");
    assert_eq!(res.status(), 200);

    let _ = shutdown_tx.send(());
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), handle).await;
    assert!(result.is_ok(), "server did not shut down in time");
}

#[tokio::test]
#[serial]
async fn full_smoke_with_wiki_api_shell() {
    let config = test_config();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let handle = tokio::spawn(run(config, ready_tx, shutdown_rx));

    let addr = tokio::time::timeout(std::time::Duration::from_secs(30), ready_rx)
        .await
        .expect("server did not become ready")
        .expect("ready channel closed");
    let url = format!("http://{}", addr);
    let client = reqwest::Client::new();

    let login = client
        .post(format!("{}/api/v1/auth/login", url))
        .json(&serde_json::json!({"email":"demo@example.com","password":"demo"}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200);
    let token = login.json::<serde_json::Value>().await.unwrap()["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    let spaces = client
        .get(format!("{}/api/v1/spaces", url))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(spaces.status(), 200);
    let body: serde_json::Value = spaces.json().await.unwrap();
    assert_eq!(body["spaces"][0]["key"], "SDLC");

    let task = client
        .get(format!("{}/api/v1/spaces/SDLC/tasks/SDLC-42", url))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(task.status(), 200);
    let body: serde_json::Value = task.json().await.unwrap();
    assert_eq!(body["task_key"], "SDLC-42");

    let _ = shutdown_tx.send(());
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), handle).await;
    assert!(result.is_ok(), "server did not shut down in time");
}
