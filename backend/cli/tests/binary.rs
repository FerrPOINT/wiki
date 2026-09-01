use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::State,
    http::{HeaderMap, Method, Request, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::{
    io::Write,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::net::TcpListener;

#[derive(Clone, Debug)]
struct RecordedRequest {
    method: Method,
    path: String,
    authorization: Option<String>,
    content_type: Option<String>,
    idempotency_key: Option<String>,
    request_id: Option<String>,
    body: Vec<u8>,
}

#[derive(Default)]
struct MockState {
    requests: Mutex<Vec<RecordedRequest>>,
}

struct MockServer {
    api_url: String,
    state: Arc<MockState>,
}

impl MockServer {
    fn requests(&self) -> Vec<RecordedRequest> {
        self.state.requests.lock().unwrap().clone()
    }
}

async fn spawn_mock_server() -> MockServer {
    let state = Arc::new(MockState::default());
    let app = Router::new()
        .fallback(record_request)
        .with_state(state.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    MockServer {
        api_url: format!("http://{addr}/api/v1"),
        state,
    }
}

async fn record_request(State(state): State<Arc<MockState>>, request: Request<Body>) -> Response {
    let method = request.method().clone();
    let path = request
        .uri()
        .path_and_query()
        .map(|value| value.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());
    let path_only = request.uri().path().to_string();
    let headers = request.headers().clone();
    let body = to_bytes(request.into_body(), 1024 * 1024).await.unwrap();
    state.requests.lock().unwrap().push(RecordedRequest {
        method,
        path,
        authorization: header_string(&headers, "authorization"),
        content_type: header_string(&headers, "content-type"),
        idempotency_key: header_string(&headers, "idempotency-key"),
        request_id: header_string(&headers, "x-request-id"),
        body: body.to_vec(),
    });

    let (status, payload) = match path_only.as_str() {
        "/api/v1/spaces" => (
            StatusCode::OK,
            json!({
                "spaces": [
                    { "key": "SDLC", "name": "SDLC Knowledge Base", "status": "active" },
                    { "key": "OPS", "name": "Operations", "status": "archived" }
                ]
            }),
        ),
        "/api/v1/spaces/SDLC%20KB" => (
            StatusCode::OK,
            json!({ "key": "SDLC KB", "name": "SDLC Knowledge Base", "status": "active" }),
        ),
        "/api/v1/spaces/SDLC/documents" => (
            StatusCode::CREATED,
            json!({ "id": "doc-from-stdin", "status": "draft" }),
        ),
        _ => (
            StatusCode::BAD_REQUEST,
            json!({
                "error": {
                    "code": "VALIDATION_ERROR",
                    "message": "Request validation failed",
                    "requestId": "req-binary",
                    "details": [{ "field": "q", "message": "too short" }]
                }
            }),
        ),
    };

    (status, Json(payload)).into_response()
}

fn header_string(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn unique_missing_markdown_path() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir()
        .join(format!("wiki-cli-missing-{nanos}.md"))
        .to_string_lossy()
        .into_owned()
}

#[tokio::test]
async fn compiled_binary_returns_non_zero_exit_for_api_error_envelope() {
    let server = spawn_mock_server().await;
    let api_url = server.api_url.clone();

    let output = tokio::task::spawn_blocking(move || {
        Command::new(env!("CARGO_BIN_EXE_wiki"))
            .arg("--api-url")
            .arg(api_url)
            .arg("search")
            .arg("query")
            .arg("x")
            .output()
            .expect("wiki binary should run")
    })
    .await
    .expect("wiki binary task should complete");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("API returned 400 Bad Request"));
    assert!(stderr.contains("VALIDATION_ERROR: Request validation failed"));
    assert!(stderr.contains("requestId=req-binary"));
    assert!(stderr.contains("details=q: too short"));

    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, Method::GET);
    assert_eq!(requests[0].path, "/api/v1/search?q=x");
}

#[tokio::test]
async fn compiled_binary_reads_markdown_from_stdin_for_doc_create() {
    let server = spawn_mock_server().await;
    let api_url = server.api_url.clone();

    let output = tokio::task::spawn_blocking(move || {
        let mut child = Command::new(env!("CARGO_BIN_EXE_wiki"))
            .arg("--api-url")
            .arg(api_url)
            .arg("--token")
            .arg("secret-token")
            .arg("doc")
            .arg("create")
            .arg("--space")
            .arg("SDLC")
            .arg("--title")
            .arg("STDIN Requirements")
            .arg("--type")
            .arg("requirements")
            .arg("--task")
            .arg("SDLC-42")
            .arg("--phase")
            .arg("testing")
            .arg("--from-file")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("wiki binary should spawn");

        let mut stdin = child.stdin.take().expect("stdin pipe should exist");
        stdin
            .write_all(b"# STDIN Requirements\n\nBody from stdin.")
            .expect("stdin write should succeed");
        drop(stdin);

        child.wait_with_output().expect("wiki binary should run")
    })
    .await
    .expect("wiki binary task should complete");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("doc-from-stdin"));

    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(request.method, Method::POST);
    assert_eq!(request.path, "/api/v1/spaces/SDLC/documents");
    assert_eq!(
        request.authorization.as_deref(),
        Some("Bearer secret-token")
    );
    assert!(
        request
            .content_type
            .as_deref()
            .is_some_and(|value| value.starts_with("application/json"))
    );
    assert!(
        request
            .idempotency_key
            .as_deref()
            .is_some_and(|value| value.starts_with("wiki-cli-write-"))
    );
    assert!(
        request
            .request_id
            .as_deref()
            .is_some_and(|value| value.starts_with("wiki-cli-request-"))
    );

    let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
    assert_eq!(body["title"], "STDIN Requirements");
    assert_eq!(body["document_type"], "requirements");
    assert_eq!(body["task_key"], "SDLC-42");
    assert_eq!(body["phase_key"], "testing");
    assert_eq!(
        body["content_markdown"],
        "# STDIN Requirements\n\nBody from stdin."
    );
}

#[tokio::test]
async fn compiled_binary_missing_markdown_file_fails_before_http() {
    let server = spawn_mock_server().await;
    let api_url = server.api_url.clone();
    let missing_path = unique_missing_markdown_path();

    let output = tokio::task::spawn_blocking(move || {
        Command::new(env!("CARGO_BIN_EXE_wiki"))
            .arg("--api-url")
            .arg(api_url)
            .arg("doc")
            .arg("create")
            .arg("--space")
            .arg("SDLC")
            .arg("--title")
            .arg("Missing file")
            .arg("--from-file")
            .arg(missing_path)
            .output()
            .expect("wiki binary should run")
    })
    .await
    .expect("wiki binary task should complete");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to read"));
    assert!(server.requests().is_empty());
}

#[tokio::test]
async fn compiled_binary_uses_env_options_and_compact_output() {
    let server = spawn_mock_server().await;
    let api_url = server.api_url.clone();

    let output = tokio::task::spawn_blocking(move || {
        Command::new(env!("CARGO_BIN_EXE_wiki"))
            .env("WIKI_API_URL", api_url)
            .env("WIKI_TOKEN", "env-token")
            .env("WIKI_OUTPUT", "compact")
            .arg("space")
            .arg("get")
            .arg("SDLC KB")
            .output()
            .expect("wiki binary should run")
    })
    .await
    .expect("wiki binary task should complete");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "key=SDLC KB | name=SDLC Knowledge Base | status=active"
    );

    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, Method::GET);
    assert_eq!(requests[0].path, "/api/v1/spaces/SDLC%20KB");
    assert_eq!(
        requests[0].authorization.as_deref(),
        Some("Bearer env-token")
    );
    assert!(requests[0].idempotency_key.is_none());
}

#[tokio::test]
async fn compiled_binary_table_output_prints_list_items() {
    let server = spawn_mock_server().await;
    let api_url = server.api_url.clone();

    let output = tokio::task::spawn_blocking(move || {
        Command::new(env!("CARGO_BIN_EXE_wiki"))
            .arg("--api-url")
            .arg(api_url)
            .arg("--output")
            .arg("table")
            .arg("space")
            .arg("list")
            .output()
            .expect("wiki binary should run")
    })
    .await
    .expect("wiki binary task should complete");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "key=SDLC | name=SDLC Knowledge Base | status=active\nkey=OPS | name=Operations | status=archived"
    );

    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, Method::GET);
    assert_eq!(requests[0].path, "/api/v1/spaces");
    assert!(requests[0].authorization.is_none());
    assert!(requests[0].idempotency_key.is_none());
}
