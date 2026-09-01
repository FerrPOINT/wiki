use axum::{
    body::{Body, to_bytes},
    http::{HeaderMap, Method, Request, StatusCode},
};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use std::{env, path::PathBuf, sync::Arc};
use tower::ServiceExt;
use uuid::Uuid;

fn test_config_with_registration(registration_enabled: bool) -> Arc<shared::AppConfig> {
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
            registration_enabled,
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

fn test_config() -> Arc<shared::AppConfig> {
    test_config_with_registration(true)
}

fn test_app_with_config(config: Arc<shared::AppConfig>) -> axum::Router {
    let ctx = Arc::new(app::WikiAppContext::new(config));
    api::router_for_memory_tests(ctx.clone()).with_state(ctx)
}

fn test_app() -> axum::Router {
    test_app_with_config(test_config())
}

fn postgres_test_config_with_registration(
    database_url: String,
    storage_dir: PathBuf,
    registration_enabled: bool,
) -> Arc<shared::AppConfig> {
    let mut cfg = (*test_config_with_registration(registration_enabled)).clone();
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

async fn postgres_test_app_with_registration(
    database_url: String,
    storage_dir: PathBuf,
    registration_enabled: bool,
) -> (axum::Router, Arc<shared::AppConfig>) {
    let config =
        postgres_test_config_with_registration(database_url, storage_dir, registration_enabled);
    let ctx = Arc::new(app::WikiAppContext::new(config.clone()));
    let storage = Arc::new(infra::LocalWikiAttachmentStorage::new(&config.storage.dir));
    let (backend, settings) = infra::connect_postgres_wiki_backend(&config, storage)
        .await
        .unwrap();
    let wiki_backend = api::routes::wiki::WikiBackend::persistent(backend, settings);
    (
        api::router_with_wiki(ctx.clone(), wiki_backend).with_state(ctx),
        config,
    )
}

async fn postgres_test_app(
    database_url: String,
    storage_dir: PathBuf,
) -> (axum::Router, Arc<shared::AppConfig>) {
    postgres_test_app_with_registration(database_url, storage_dir, true).await
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
    call_body(app, method, path, token, "application/json", body).await
}

async fn call_body(
    app: &axum::Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    content_type: &str,
    body: Vec<u8>,
) -> (StatusCode, Value) {
    let mut request = Request::builder().method(method).uri(path);
    request = request.header("content-type", content_type);
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

async fn call_binary(
    app: &axum::Router,
    method: Method,
    path: &str,
    token: Option<&str>,
) -> (StatusCode, HeaderMap, Vec<u8>) {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    (status, headers, bytes)
}

async fn upload_test_file(
    app: &axum::Router,
    token: &str,
    file_name: &str,
    content_type: &str,
    bytes: &[u8],
) -> (StatusCode, Value) {
    let boundary = "wiki-test-boundary";
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\nContent-Type: {content_type}\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    call_body(
        app,
        Method::POST,
        "/api/v1/attachments",
        Some(token),
        &format!("multipart/form-data; boundary={boundary}"),
        body,
    )
    .await
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
async fn wiki_persistent_backend_requires_database_url() {
    let config = test_config();
    let storage_dir = env::temp_dir().join(format!("wiki-api-test-{}", Uuid::now_v7()));
    let storage = Arc::new(infra::LocalWikiAttachmentStorage::new(storage_dir));

    let backend_result = infra::connect_postgres_wiki_backend(&config, storage).await;
    let err = match backend_result {
        Ok(_) => panic!("persistent backend must reject an empty database URL"),
        Err(err) => err,
    };

    assert!(matches!(
        err,
        shared::AppError::InvalidInput(message) if message.contains("WIKI_DATABASE__URL")
    ));
}

#[tokio::test]
async fn wiki_register_respects_instance_registration_setting() {
    let app = test_app_with_config(test_config_with_registration(false));

    let (status, body) = call(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        None,
        Some(json!({
            "email": "new@example.com",
            "username": "new",
            "password": "new-password",
            "name": "New User"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "FORBIDDEN");
    assert_eq!(body["error"]["message"], "forbidden");

    let (status, login) = call(
        &app,
        Method::POST,
        "/api/v1/auth/login",
        None,
        Some(json!({ "email": "demo@example.com", "password": "demo" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(login["access_token"].is_string());
}

#[tokio::test]
async fn wiki_settings_are_admin_only_and_config_backed() {
    let app = test_app_with_config(test_config_with_registration(false));

    let (status, login) = call(
        &app,
        Method::POST,
        "/api/v1/auth/login",
        None,
        Some(json!({ "email": "demo@example.com", "password": "demo" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let admin_token = login["access_token"].as_str().unwrap();

    let (status, settings) = call(
        &app,
        Method::GET,
        "/api/v1/settings",
        Some(admin_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(settings["instance_name"], "Wiki");
    assert_eq!(settings["api_base_path"], "/api/v1");
    assert_eq!(settings["default_space_key"], "SDLC");
    assert_eq!(settings["registration_enabled"], false);
    assert_eq!(settings["storage_backend"], "local");
    assert_eq!(settings["search_backend"], "PostgreSQL FTS");
    assert_eq!(settings["max_upload_bytes"], 25 * 1024 * 1024);

    let app = test_app();
    let (status, user) = call(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        None,
        Some(json!({
            "email": "regular@example.com",
            "username": "regular",
            "password": "regular-password",
            "name": "Regular User"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let user_token = user["access_token"].as_str().unwrap();
    let (status, body) = call(
        &app,
        Method::GET,
        "/api/v1/settings",
        Some(user_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "FORBIDDEN");
}

#[tokio::test]
async fn wiki_memory_authz_and_audit_align_with_mvp_contract() {
    let app = test_app();
    let (status, login) = call(
        &app,
        Method::POST,
        "/api/v1/auth/login",
        None,
        Some(json!({ "email": "demo@example.com", "password": "demo" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let admin_token = login["access_token"].as_str().unwrap();
    let suffix = Uuid::now_v7().simple().to_string();
    let short = &suffix[..12];

    let (status, regular) = call(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        None,
        Some(json!({
            "email": format!("viewer-{short}@example.com"),
            "username": format!("viewer-{short}"),
            "password": "viewer-password",
            "name": "Viewer"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let viewer_token = regular["access_token"].as_str().unwrap();
    let viewer_id = regular["user_id"].as_str().unwrap();

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
        "/api/v1/spaces/SDLC",
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, search) = call(
        &app,
        Method::GET,
        "/api/v1/search?q=Wiki",
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(search["results"].as_array().unwrap().len(), 0);

    let (status, _) = call(
        &app,
        Method::GET,
        "/api/v1/search?q=Wiki&space=SDLC",
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = call(
        &app,
        Method::GET,
        "/api/v1/audit-log",
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/spaces",
        Some(viewer_token),
        Some(json!({
            "key": format!("NOPE-{short}"),
            "name": "Viewer must not create spaces"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, member) = call(
        &app,
        Method::PUT,
        &format!("/api/v1/spaces/SDLC/members/{viewer_id}"),
        Some(admin_token),
        Some(json!({ "role": "viewer" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(member["role"], "viewer");

    let (status, space) = call(
        &app,
        Method::GET,
        "/api/v1/spaces/SDLC",
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(space["key"], "SDLC");

    let (status, document) = call(
        &app,
        Method::GET,
        "/api/v1/documents/product-requirements",
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(document["slug"], "product-requirements");

    let (status, _) = call(
        &app,
        Method::PUT,
        "/api/v1/documents/product-requirements/draft",
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
        Some(viewer_token),
        Some(json!({
            "space": "SDLC",
            "document_id": "product-requirements",
            "title": "Viewer must not add evidence",
            "evidence_type": "external_url",
            "url": "https://ci.local/jobs/forbidden"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, writable_space) = call(
        &app,
        Method::POST,
        "/api/v1/spaces",
        Some(admin_token),
        Some(json!({
            "key": format!("ARCH-{short}"),
            "name": "Archived write check"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let archived_space_key = writable_space["key"].as_str().unwrap();

    let (status, archived_space) = call(
        &app,
        Method::POST,
        &format!("/api/v1/spaces/{archived_space_key}/archive"),
        Some(admin_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(archived_space["status"], "archived");

    let (status, _) = call(
        &app,
        Method::POST,
        &format!("/api/v1/spaces/{archived_space_key}/documents"),
        Some(admin_token),
        Some(json!({
            "title": "Archived space document",
            "slug": format!("archived-space-document-{short}"),
            "document_type": "page",
            "content_markdown": "# Should not be created"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(admin_token),
        Some(json!({
            "space": archived_space_key,
            "task_key": format!("ARCH-{short}"),
            "title": "Archived space evidence",
            "evidence_type": "external_url",
            "url": "https://ci.local/jobs/archived"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, document) = call(
        &app,
        Method::POST,
        "/api/v1/spaces/SDLC/documents",
        Some(admin_token),
        Some(json!({
            "title": format!("Audit document {short}"),
            "slug": format!("audit-document-{short}"),
            "document_type": "requirements",
            "content_markdown": "# Audit document\n\nRecords should appear in audit log."
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let document_id = document["id"].as_str().unwrap();

    let (status, _) = call(
        &app,
        Method::POST,
        &format!("/api/v1/documents/{document_id}/publish"),
        Some(admin_token),
        Some(json!({ "summary": "Audit publish" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let attachment_bytes = b"memory file evidence bytes";
    let (status, attachment) = upload_test_file(
        &app,
        admin_token,
        &format!("memory-evidence-{short}.txt"),
        "text/plain",
        attachment_bytes,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let attachment_id = attachment["id"].as_str().unwrap();

    let (status, file_evidence) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(admin_token),
        Some(json!({
            "space": "SDLC",
            "document_id": document_id,
            "title": "Audit file evidence",
            "evidence_type": "uploaded_file",
            "attachment_id": attachment_id
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(file_evidence["attachment_id"], attachment_id);
    assert_eq!(file_evidence["checksum"], attachment["checksum"]);

    let (status, _, downloaded) = call_binary(
        &app,
        Method::GET,
        &format!("/api/v1/attachments/{attachment_id}/download"),
        Some(viewer_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(downloaded, attachment_bytes);

    let (status, member) = call(
        &app,
        Method::PUT,
        &format!("/api/v1/spaces/SDLC/members/{viewer_id}"),
        Some(admin_token),
        Some(json!({ "role": "editor" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(member["role"], "editor");

    let revoked_attachment_bytes = b"claimed attachment should follow space access";
    let (status, revoked_attachment) = upload_test_file(
        &app,
        viewer_token,
        &format!("revoked-editor-evidence-{short}.txt"),
        "text/plain",
        revoked_attachment_bytes,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let revoked_attachment_id = revoked_attachment["id"].as_str().unwrap();

    let (status, revoked_file_evidence) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(viewer_token),
        Some(json!({
            "space": "SDLC",
            "document_id": document_id,
            "title": "Claimed attachment access check",
            "evidence_type": "uploaded_file",
            "attachment_id": revoked_attachment_id
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        revoked_file_evidence["checksum"],
        revoked_attachment["checksum"]
    );
    let revoked_evidence_id = revoked_file_evidence["id"].as_str().unwrap();

    let (status, _) = call(
        &app,
        Method::DELETE,
        &format!("/api/v1/spaces/SDLC/members/{viewer_id}"),
        Some(admin_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _, _) = call_binary(
        &app,
        Method::GET,
        &format!("/api/v1/attachments/{revoked_attachment_id}/download"),
        Some(viewer_token),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _, admin_downloaded) = call_binary(
        &app,
        Method::GET,
        &format!("/api/v1/attachments/{revoked_attachment_id}/download"),
        Some(admin_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(admin_downloaded, revoked_attachment_bytes);

    let (status, evidence) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(admin_token),
        Some(json!({
            "space": "SDLC",
            "document_id": document_id,
            "task_key": format!("AUD-{short}"),
            "title": "Audit evidence",
            "evidence_type": "external_url",
            "url": "https://ci.local/jobs/audit"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let evidence_id = evidence["id"].as_str().unwrap();

    let (status, audit) = call(
        &app,
        Method::GET,
        "/api/v1/audit-log",
        Some(admin_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let entries = audit["entries"].as_array().unwrap();
    assert!(
        entries.iter().any(|entry| {
            entry["action"] == "space.member_upsert" && entry["entity_id"] == "SDLC"
        })
    );
    assert!(entries.iter().any(|entry| {
        entry["action"] == "document.create" && entry["entity_id"] == document_id
    }));
    assert!(entries.iter().any(|entry| {
        entry["action"] == "document.publish" && entry["entity_id"] == document_id
    }));
    assert!(entries.iter().any(|entry| {
        entry["action"] == "attachment.upload" && entry["entity_id"] == attachment_id
    }));
    assert!(entries.iter().any(|entry| {
        entry["action"] == "evidence.create" && entry["entity_id"] == evidence_id
    }));
    assert!(
        entries.iter().any(|entry| {
            entry["action"] == "space.member_delete" && entry["entity_id"] == "SDLC"
        })
    );
    assert!(entries.iter().any(|entry| {
        entry["action"] == "evidence.create" && entry["entity_id"] == revoked_evidence_id
    }));
}

#[tokio::test]
async fn wiki_document_move_rejects_descendant_parent() {
    let app = test_app();
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
    let suffix = Uuid::now_v7().simple().to_string();
    let root_slug = format!("tree-root-{}", &suffix[..12]);
    let child_slug = format!("tree-child-{}", &suffix[..12]);

    let (status, root) = call(
        &app,
        Method::POST,
        "/api/v1/spaces/SDLC/documents",
        Some(token),
        Some(json!({
            "title": "Tree Root",
            "slug": root_slug,
            "document_type": "page",
            "content_markdown": "# Tree Root"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let root_id = root["id"].as_str().unwrap();

    let (status, child) = call(
        &app,
        Method::POST,
        "/api/v1/spaces/SDLC/documents",
        Some(token),
        Some(json!({
            "title": "Tree Child",
            "slug": child_slug,
            "parent_id": root_id,
            "document_type": "page",
            "content_markdown": "# Tree Child"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let child_id = child["id"].as_str().unwrap();

    let (status, error) = call(
        &app,
        Method::POST,
        &format!("/api/v1/documents/{root_id}/move"),
        Some(token),
        Some(json!({ "parent_id": child_id })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(
        error["error"]["message"],
        "document cannot be moved under its descendant"
    );

    let (status, root_after) = call(
        &app,
        Method::GET,
        &format!("/api/v1/documents/{root_id}"),
        Some(token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(root_after["parent_id"].is_null());
}

#[tokio::test]
async fn wiki_postgres_register_respects_instance_registration_setting() {
    let Ok(database_url) = env::var("WIKI_TEST_DATABASE_URL") else {
        eprintln!("skipping postgres registration test: WIKI_TEST_DATABASE_URL is not set");
        return;
    };
    reset_postgres(&database_url).await;
    let storage_dir = env::temp_dir().join(format!("wiki-api-test-{}", Uuid::now_v7()));
    let (app, _) = postgres_test_app_with_registration(database_url, storage_dir, false).await;

    let (status, body) = call(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        None,
        Some(json!({
            "email": "postgres-new@example.com",
            "username": "postgres-new",
            "password": "new-password",
            "name": "Postgres New User"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "FORBIDDEN");
    assert_eq!(body["error"]["message"], "forbidden");

    let token = login_admin(&app).await;
    assert!(!token.is_empty());
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

    let (status, settings) = call(&app, Method::GET, "/api/v1/settings", Some(token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(settings["instance_name"], "Wiki");
    assert_eq!(settings["registration_enabled"], true);
    assert_eq!(settings["default_space_key"], "SDLC");

    let (status, spaces) = call(&app, Method::GET, "/api/v1/spaces", Some(token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        spaces["spaces"]
            .as_array()
            .unwrap()
            .iter()
            .any(|space| space["key"] == "SDLC")
    );

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

    let (status, document_evidence) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(token),
        Some(json!({
            "space": "SDLC",
            "document_id": document_id,
            "title": "Document evidence",
            "evidence_type": "external_url",
            "url": "https://ci.local/jobs/wiki-document-smoke"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(document_evidence["document_id"], document_id);

    let (status, document_evidence_list) = call(
        &app,
        Method::GET,
        &format!("/api/v1/evidence?document_id={document_id}"),
        Some(token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        document_evidence_list["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["title"] == "Document evidence")
    );

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

    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(token),
        Some(json!({
            "space": "SDLC",
            "task_key": "SDLC-99",
            "title": "External checksum must not be accepted",
            "evidence_type": "external_url",
            "url": "https://ci.local/jobs/wiki-smoke",
            "checksum": "sha256:external"
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
            "title": "Uploaded checksum must not be accepted",
            "evidence_type": "uploaded_file",
            "attachment_id": "00000000-0000-0000-0000-000000000001",
            "checksum": "sha256:client"
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
    let suffix = Uuid::now_v7().simple().to_string();
    let short = &suffix[..12];

    let (status, spaces) = call(&app, Method::GET, "/api/v1/spaces", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(spaces["spaces"][0]["key"], "SDLC");

    let archived_key = format!("PGARCH-{short}");
    let (status, archived_space) = call(
        &app,
        Method::POST,
        "/api/v1/spaces",
        Some(&token),
        Some(json!({
            "key": archived_key,
            "name": "Postgres archived write check"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let archived_key = archived_space["key"].as_str().unwrap();

    let (status, archived_space) = call(
        &app,
        Method::POST,
        &format!("/api/v1/spaces/{archived_key}/archive"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(archived_space["status"], "archived");

    let (status, _) = call(
        &app,
        Method::POST,
        &format!("/api/v1/spaces/{archived_key}/documents"),
        Some(&token),
        Some(json!({
            "title": "Postgres archived space document",
            "slug": format!("postgres-archived-space-document-{short}"),
            "document_type": "page",
            "content_markdown": "# Should not be created"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(&token),
        Some(json!({
            "space": archived_key,
            "task_key": format!("PGARCH-{short}"),
            "title": "Postgres archived space evidence",
            "evidence_type": "external_url",
            "url": "https://ci.local/jobs/postgres-archived"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

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

    let (status, child_document) = call(
        &app,
        Method::POST,
        "/api/v1/spaces/SDLC/documents",
        Some(&token),
        Some(json!({
            "title": "Persistent Child",
            "slug": "persistent-child",
            "parent_id": document_id,
            "document_type": "page",
            "content_markdown": "# Persistent Child"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let child_document_id = child_document["id"].as_str().unwrap().to_string();

    let (status, cycle_error) = call(
        &app,
        Method::POST,
        &format!("/api/v1/documents/{document_id}/move"),
        Some(&token),
        Some(json!({ "parent_id": child_document_id })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        cycle_error["error"]["message"],
        "document cannot be moved under its descendant"
    );

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

    let (status, stranger) = call(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        None,
        Some(json!({
            "email": "stranger@example.com",
            "username": "stranger",
            "password": "stranger-password",
            "name": "Stranger"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let stranger_token = stranger["access_token"].as_str().unwrap();

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

    let attachment_bytes = b"wiki attachment bytes";
    let (status, attachment) = upload_test_file(
        &app,
        &token,
        "test evidence.txt",
        "text/plain",
        attachment_bytes,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let attachment_id = attachment["id"].as_str().unwrap();
    assert_eq!(attachment["file_name"], "test evidence.txt");
    assert_eq!(attachment["content_type"], "text/plain");
    assert_eq!(attachment["size_bytes"], attachment_bytes.len());

    let (status, _) = call(
        &app,
        Method::GET,
        &format!("/api/v1/attachments/{attachment_id}"),
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _, _) = call_binary(
        &app,
        Method::GET,
        &format!("/api/v1/attachments/{attachment_id}/download"),
        Some(stranger_token),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, missing_file_attachment) =
        upload_test_file(&app, &token, "missing.txt", "text/plain", b"deleted later").await;
    assert_eq!(status, StatusCode::CREATED);
    let missing_file_attachment_id = missing_file_attachment["id"].as_str().unwrap();
    tokio::fs::remove_file(
        storage_dir
            .join("attachments")
            .join(missing_file_attachment_id)
            .join("missing.txt"),
    )
    .await
    .unwrap();
    let (status, _, _) = call_binary(
        &app,
        Method::GET,
        &format!("/api/v1/attachments/{missing_file_attachment_id}/download"),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

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

    let (status, viewer_attachment) = upload_test_file(
        &app,
        viewer_token,
        "viewer-staged.txt",
        "text/plain",
        b"viewer staged bytes",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let viewer_attachment_id = viewer_attachment["id"].as_str().unwrap();

    let (status, own_staged_attachment) = call(
        &app,
        Method::GET,
        &format!("/api/v1/attachments/{viewer_attachment_id}"),
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(own_staged_attachment["id"], viewer_attachment_id);

    let (status, _) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(viewer_token),
        Some(json!({
            "space": "SDLC",
            "document_id": document_id.clone(),
            "title": "Viewer must not claim file evidence",
            "evidence_type": "uploaded_file",
            "attachment_id": viewer_attachment_id
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

    let (status, file_evidence) = call(
        &app,
        Method::POST,
        "/api/v1/evidence",
        Some(&token),
        Some(json!({
            "space": "SDLC",
            "document_id": document_id.clone(),
            "task_key": "SDLC-777",
            "phase_key": "testing",
            "title": "Persistent file evidence",
            "evidence_type": "uploaded_file",
            "attachment_id": attachment_id
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(file_evidence["attachment_id"], attachment_id);
    assert_eq!(file_evidence["checksum"], attachment["checksum"]);

    let (status, attached_metadata) = call(
        &app,
        Method::GET,
        &format!("/api/v1/attachments/{attachment_id}"),
        Some(viewer_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(attached_metadata["id"], attachment_id);

    let (status, headers, downloaded) = call_binary(
        &app,
        Method::GET,
        &format!("/api/v1/attachments/{attachment_id}/download"),
        Some(viewer_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(downloaded, attachment_bytes);
    assert_eq!(
        headers
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/plain")
    );

    let (status, _) = call(
        &app,
        Method::GET,
        &format!("/api/v1/attachments/{attachment_id}"),
        Some(stranger_token),
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
    assert_eq!(task["evidence_count"], 2);
    assert!(
        task["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["evidence_type"] == "uploaded_file")
    );

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

    let (status, phrase_search) = call(
        &app,
        Method::GET,
        "/api/v1/search?q=Postgres%20Wiki%20document&space=SDLC",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        phrase_search["results"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == persisted_document_id)
    );

    let (status, substring_search) = call(
        &app,
        Method::GET,
        "/api/v1/search?q=gres&space=SDLC",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        !substring_search["results"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == persisted_document_id)
    );
}
