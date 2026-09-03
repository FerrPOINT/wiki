use std::sync::Arc;

use serial_test::serial;
use server::{run, run_with_wiki_backend};
use shared::{AppConfig, AuthConfig, EmailConfig, RuntimeEnvironment, ServerConfig};

fn test_config() -> Arc<AppConfig> {
    Arc::new(AppConfig {
        environment: RuntimeEnvironment::Test,
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
            registration_enabled: true,
            refresh_cookie_name: "refresh_token".to_string(),
            refresh_cookie_secure: true,
            refresh_cookie_same_site: "Lax".to_string(),
            refresh_cookie_domain: None,
            refresh_cookie_path: "/api/v1/auth".to_string(),
        },
        storage: shared::StorageConfig::default(),
        maintenance: shared::MaintenanceConfig::default(),
        email: EmailConfig::default(),
        bootstrap: shared::BootstrapConfig::default(),
    })
}

async fn run_test_server(
    config: Arc<AppConfig>,
    ready: tokio::sync::oneshot::Sender<std::net::SocketAddr>,
    shutdown: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), shared::AppError> {
    let wiki_backend = api::routes::wiki::WikiBackend::memory_from_config(&config);
    run_with_wiki_backend(config, wiki_backend, ready, shutdown).await
}

async fn assert_server_stops(handle: tokio::task::JoinHandle<Result<(), shared::AppError>>) {
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), handle)
        .await
        .expect("server did not shut down in time")
        .expect("server task panicked");
    result.expect("server returned error");
}

#[tokio::test]
#[serial]
async fn production_run_requires_database_url() {
    let config = test_config();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let result = run(config, ready_tx, shutdown_rx).await;

    assert!(matches!(
        result,
        Err(shared::AppError::InvalidInput(message))
            if message.contains("WIKI_DATABASE__URL")
    ));
    assert!(
        ready_rx.await.is_err(),
        "production server must not signal readiness without PostgreSQL"
    );
}

#[tokio::test]
#[serial]
async fn server_starts_and_serves_health() {
    let config = test_config();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let handle = tokio::spawn(run_test_server(config, ready_tx, shutdown_rx));

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

    let readiness = client
        .get(format!("http://{}/api/v1/health/ready", addr))
        .send()
        .await
        .expect("readiness request failed");
    assert_eq!(readiness.status(), 200);
    assert_eq!(readiness.text().await.unwrap(), "ready");

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
    assert_server_stops(handle).await;
}

#[tokio::test]
#[serial]
async fn full_smoke_with_wiki_api_shell() {
    let config = test_config();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let handle = tokio::spawn(run_test_server(config, ready_tx, shutdown_rx));

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
    assert_server_stops(handle).await;
}
