use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;

fn test_config() -> Arc<shared::AppConfig> {
    Arc::new(shared::AppConfig {
        environment: shared::RuntimeEnvironment::Test,
        database: shared::DatabaseConfig::default(),
        server: shared::ServerConfig {
            general_rate_burst: 1,
            general_rate_period_secs: 60,
            ..shared::ServerConfig::default()
        },
        auth: shared::AuthConfig {
            jwt_secret: "test-secret".to_string(),
            access_token_ttl_minutes: 15,
            refresh_token_ttl_days: 7,
            registration_enabled: true,
            refresh_cookie_name: "refresh_token".to_string(),
            refresh_cookie_secure: false,
            refresh_cookie_same_site: "Lax".to_string(),
            refresh_cookie_domain: None,
            refresh_cookie_path: "/api/v1/auth".to_string(),
        },
        storage: shared::StorageConfig::default(),
        maintenance: shared::MaintenanceConfig::default(),
        email: shared::EmailConfig::default(),
        bootstrap: shared::BootstrapConfig::default(),
    })
}

#[tokio::test]
async fn health_probes_are_not_consumed_by_the_general_api_rate_limit() {
    let ctx = Arc::new(app::WikiAppContext::new(test_config()));
    let app = api::router_for_memory_tests(ctx.clone()).with_state(ctx);

    for path in ["/api/v1/health", "/api/v1/health/ready", "/api/v1/health"] {
        let response = app
            .clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "{path} must stay probe-safe"
        );
    }
}

#[tokio::test]
async fn general_limit_replenishes_its_burst_within_the_configured_window() {
    let mut config = (*test_config()).clone();
    config.server.general_rate_burst = 2;
    config.server.general_rate_period_secs = 1;
    let ctx = Arc::new(app::WikiAppContext::new(Arc::new(config)));
    let app = api::router_for_memory_tests(ctx.clone()).with_state(ctx);

    for expected in [
        StatusCode::UNAUTHORIZED,
        StatusCode::UNAUTHORIZED,
        StatusCode::TOO_MANY_REQUESTS,
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::get("/api/v1/users/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
    }

    tokio::time::sleep(std::time::Duration::from_millis(750)).await;
    let response = app
        .oneshot(
            Request::get("/api/v1/users/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
