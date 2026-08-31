use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use std::{env, path::PathBuf, sync::Arc};
use tower::ServiceExt;
use uuid::Uuid;

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
        bootstrap: shared::BootstrapConfig::default(),
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

fn postgres_test_config(database_url: String, storage_dir: PathBuf) -> Arc<shared::AppConfig> {
    let mut cfg = (*test_config()).clone();
    cfg.database.url = database_url;
    cfg.database.max_connections = 5;
    cfg.database.min_connections = 1;
    cfg.storage = shared::StorageConfig {
        dir: storage_dir.to_string_lossy().into_owned(),
        max_upload_bytes: 1024 * 1024,
    };
    cfg.bootstrap = shared::BootstrapConfig {
        admin_email: Some("admin@example.com".to_string()),
        admin_username: Some("admin".to_string()),
        admin_password: Some("admin-password".to_string()),
        admin_display_name: Some("Администратор Wiki".to_string()),
    };
    Arc::new(cfg)
}

async fn postgres_test_app(
    database_url: String,
    storage_dir: PathBuf,
) -> (axum::Router, Arc<shared::AppConfig>) {
    let config = postgres_test_config(database_url, storage_dir);
    let ctx = Arc::new(app::AppContext::new(
        config.clone(),
        Arc::new(domain::Repositories::default()),
        Arc::new(domain::InMemoryStorage::default()),
    ));
    let wiki_backend = api::routes::wiki::WikiBackend::from_config(&config)
        .await
        .unwrap();
    (
        api::router_with_wiki(ctx.clone(), wiki_backend).with_state(ctx),
        config,
    )
}

async fn reset_postgres(database_url: &str) {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await
        .unwrap();
    sqlx::query("DROP SCHEMA public CASCADE")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE SCHEMA public")
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;
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
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| json!({ "raw": String::from_utf8_lossy(&bytes) }))
    };
    (status, value)
}

async fn login_admin(app: &axum::Router) -> String {
    let (status, login) = call(
        app,
        Method::POST,
        "/api/v1/auth/login",
        None,
        Some(json!({ "email": "admin@example.com", "password": "admin-password" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    login["access_token"].as_str().unwrap().to_string()
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

#[tokio::test]
async fn wiki_postgres_routes_persist_across_router_rebuilds() {
    let Ok(database_url) = env::var("WIKI_TEST_DATABASE_URL") else {
        eprintln!("skipping postgres persistence test: WIKI_TEST_DATABASE_URL is not set");
        return;
    };
    reset_postgres(&database_url).await;
    let storage_dir = env::temp_dir().join(format!("wiki-api-test-{}", Uuid::now_v7()));
    let (app, _) = postgres_test_app(database_url.clone(), storage_dir.clone()).await;
    let token = login_admin(&app).await;

    let (status, spaces) = call(&app, Method::GET, "/api/v1/spaces", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(spaces["spaces"][0]["key"], "SDLC");

    let (status, document) = call(
        &app,
        Method::POST,
        "/api/v1/spaces/SDLC/documents",
        Some(&token),
        Some(json!({
            "title": "Persistent Requirements",
            "document_type": "requirements",
            "task_key": "SDLC-777",
            "phase_key": "testing",
            "content_markdown": "# Persistent Requirements\n\nPostgres-backed Wiki document"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let document_id = document["id"].as_str().unwrap().to_string();

    let (status, revision) = call(
        &app,
        Method::POST,
        &format!("/api/v1/documents/{document_id}/publish"),
        Some(&token),
        Some(json!({ "summary": "Postgres publish" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(revision["version"], 1);

    let (status, outsider) = call(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        None,
        Some(json!({
            "email": "viewer@example.com",
            "username": "viewer",
            "password": "viewer-password",
            "name": "Viewer"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let viewer_token = outsider["access_token"].as_str().unwrap();
    let viewer_id = outsider["user_id"].as_str().unwrap();

    let (status, spaces) = call(
        &app,
        Method::GET,
        "/api/v1/spaces",
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(spaces["spaces"].as_array().unwrap().len(), 0);

    let (status, _) = call(
        &app,
        Method::GET,
        &format!("/api/v1/documents/{document_id}"),
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, search) = call(
        &app,
        Method::GET,
        "/api/v1/search?q=Persistent",
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(search["results"].as_array().unwrap().len(), 0);

    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/spaces",
        Some(viewer_token),
        Some(json!({
            "key": "PRIVATE",
            "name": "Private space"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = call(
        &app,
        Method::GET,
        "/api/v1/search?q=Persistent&space=SDLC",
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = call(
        &app,
        Method::PUT,
        &format!("/api/v1/spaces/SDLC/members/{viewer_id}"),
        Some(&token),
        Some(json!({ "role": "viewer" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, readable_document) = call(
        &app,
        Method::GET,
        &format!("/api/v1/documents/{document_id}"),
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(readable_document["id"], document_id);

    let (status, visible_search) = call(
        &app,
        Method::GET,
        "/api/v1/search?q=Persistent&space=SDLC",
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        visible_search["results"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == document_id)
    );

    let (status, _) = call(
        &app,
        Method::PUT,
        "/api/v1/spaces/SDLC",
        Some(viewer_token),
        Some(json!({ "name": "Viewer must not rename space" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = call(
        &app,
        Method::PUT,
        &format!("/api/v1/documents/{document_id}/draft"),
        Some(viewer_token),
        Some(json!({
            "title": "Viewer must not edit",
            "content_markdown": "# Viewer must not edit"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(viewer_token),
        Some(json!({
            "space": "SDLC",
            "document_id": document_id.clone(),
            "title": "Viewer must not add evidence",
            "evidence_type": "external_url",
            "url": "https://ci.local/jobs/forbidden"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = call(
        &app,
        Method::GET,
        "/api/v1/spaces/SDLC/members",
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(&token),
        Some(json!({
            "space": "SDLC",
            "document_id": document_id,
            "task_key": "SDLC-777",
            "phase_key": "testing",
            "title": "Persistent evidence",
            "evidence_type": "external_url",
            "url": "https://ci.local/jobs/wiki-postgres"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    drop(app);
    let (app, _) = postgres_test_app(database_url, storage_dir).await;
    let token = login_admin(&app).await;

    let (status, task) = call(
        &app,
        Method::GET,
        "/api/v1/spaces/SDLC/tasks/SDLC-777",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(task["document_count"], 1);
    assert_eq!(task["evidence_count"], 1);

    let persisted_document_id = task["documents"][0]["id"].as_str().unwrap();
    let (status, revisions) = call(
        &app,
        Method::GET,
        &format!("/api/v1/documents/{persisted_document_id}/revisions"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(revisions["revisions"].as_array().unwrap().len(), 1);

    let (status, search) = call(
        &app,
        Method::GET,
        "/api/v1/search?q=Postgres&space=SDLC",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        search["results"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["title"] == "Persistent Requirements")
    );
}
