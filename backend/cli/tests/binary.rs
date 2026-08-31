use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::{
    process::Command,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;

#[derive(Default)]
struct MockState {
    requests: Mutex<Vec<String>>,
}

struct MockServer {
    api_url: String,
    state: Arc<MockState>,
}

impl MockServer {
    fn requests(&self) -> Vec<String> {
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
    let path = request
        .uri()
        .path_and_query()
        .map(|value| value.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());
    state.requests.lock().unwrap().push(path);

    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": {
                "code": "VALIDATION_ERROR",
                "message": "Request validation failed",
                "requestId": "req-binary",
                "details": [{ "field": "q", "message": "too short" }]
            }
        })),
    )
        .into_response()
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
    assert_eq!(server.requests(), vec!["/api/v1/search?q=x"]);
}
