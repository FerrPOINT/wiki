use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

fn test_config() -> Arc<shared::AppConfig> {
    Arc::new(shared::AppConfig {
        database: shared::DatabaseConfig::default(),
        server: shared::ServerConfig {
            auth_rate_burst: 100,
            general_rate_burst: 1000,
            ..shared::ServerConfig::default()
        },
        auth: shared::AuthConfig {
            jwt_secret: "test-secret".to_string(),
            access_token_ttl_minutes: 15,
            refresh_token_ttl_days: 7,
            refresh_cookie_name: "refresh_token".to_string(),
            refresh_cookie_secure: false,
            refresh_cookie_same_site: "Lax".to_string(),
            refresh_cookie_domain: None,
            refresh_cookie_path: "/api/v1/auth".to_string(),
        },
        storage: shared::StorageConfig::default(),
        email: shared::EmailConfig::default(),
    })
}

fn test_app() -> axum::Router {
    let ctx = Arc::new(app::AppContext::new(
        test_config(),
        Arc::new(domain::Repositories::default()),
        Arc::new(domain::InMemoryStorage::default()),
    ));
    api::router(ctx.clone()).with_state(ctx)
}

async fn call(
    app: &axum::Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let body = body.map_or_else(Vec::new, |value| serde_json::to_vec(&value).unwrap());
    let mut request = Request::builder().method(method).uri(path);
    request = request.header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body)).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value = if bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, value)
}

#[tokio::test]
async fn wiki_mvp_routes_cover_public_contract() {
    let app = test_app();

    let (status, _) = call(&app, Method::GET, "/api/v1/health", None, None).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = call(&app, Method::GET, "/api/v1/spaces", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, login) = call(
        &app,
        Method::POST,
        "/api/v1/auth/login",
        None,
        Some(json!({ "email": "demo@example.com", "password": "demo" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let token = login["access_token"].as_str().unwrap();

    let (status, spaces) = call(&app, Method::GET, "/api/v1/spaces", Some(token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(spaces["spaces"][0]["key"], "SDLC");

    let (status, document) = call(
        &app,
        Method::POST,
        "/api/v1/spaces/SDLC/documents",
        Some(token),
        Some(json!({
            "title": "Smoke Requirements",
            "document_type": "requirements",
            "task_key": "SDLC-99",
            "phase_key": "testing",
            "content_markdown": "# Smoke Requirements\n\nauthorization and publishing"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let document_id = document["id"].as_str().unwrap();

    let (status, revision) = call(
        &app,
        Method::POST,
        &format!("/api/v1/documents/{document_id}/publish"),
        Some(token),
        Some(json!({ "summary": "Smoke publish" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(revision["version"], 1);

    let (status, task) = call(
        &app,
        Method::GET,
        "/api/v1/spaces/SDLC/tasks/SDLC-99",
        Some(token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(task["document_count"], 1);

    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(token),
        Some(json!({
            "space": "SDLC",
            "task_key": "SDLC-99",
            "phase_key": "testing",
            "title": "Smoke evidence",
            "evidence_type": "external_url",
            "url": "https://ci.local/jobs/wiki-smoke"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(token),
        Some(json!({
            "space": "SDLC",
            "task_key": "SDLC-99",
            "title": "Invalid evidence",
            "evidence_type": "manual_check",
            "url": "https://ci.local/jobs/wiki-smoke"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(token),
        Some(json!({
            "space": "SDLC",
            "task_key": "SDLC-99",
            "title": "Mixed evidence payload",
            "evidence_type": "external_url",
            "url": "https://ci.local/jobs/wiki-smoke",
            "attachment_id": "00000000-0000-0000-0000-000000000001"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(token),
        Some(json!({
            "space": "SDLC",
            "task_key": "SDLC-99",
            "title": "Missing attachment",
            "evidence_type": "uploaded_file",
            "url": "https://ci.local/jobs/wiki-smoke"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, phase) = call(
        &app,
        Method::GET,
        "/api/v1/spaces/SDLC/phases/testing",
        Some(token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(phase["evidence_count"], 1);

    let (status, search) = call(
        &app,
        Method::GET,
        "/api/v1/search?q=authorization&space=SDLC",
        Some(token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        search["results"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == document_id)
    );
}
