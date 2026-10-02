use axum::{
    Router,
    body::{Body, to_bytes},
    extract::State,
    http::{Request, StatusCode},
    response::Response,
};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    io::Write,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};

#[tokio::test]
async fn evidence_file_keys_survive_partial_failure_and_process_retries() {
    let server = Server::start(vec![
        (201, json!({"id":"attachment"})),
        (500, json!({"error":{"message":"temporary failure"}})),
        (201, json!({"id":"attachment"})),
        (201, json!({"id":"evidence","attachment_id":"attachment"})),
    ])
    .await;
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("evidence.txt");
    std::fs::write(&file, b"evidence").unwrap();
    let parent = "k".repeat(128);
    let padded = format!(" {parent} ");
    let args = [
        "--idempotency-key",
        &padded,
        "evidence",
        "add-file",
        "--space",
        "CLI",
        "--title",
        "Evidence",
        "--file",
        file.to_str().unwrap(),
    ];
    let failed = server.run(&args, None).await;
    assert!(!failed.status.success());
    assert!(failed.stdout.is_empty());
    let replay = success(&server.run(&args, None).await);
    assert_eq!(replay["attachment_id"], "attachment");
    let requests = server.requests();
    assert_eq!(requests.len(), 4);
    assert_eq!(requests[0].path, "/api/v1/attachments");
    assert_eq!(requests[1].path, "/api/v1/evidence");
    let upload = requests[0].key.as_deref().unwrap();
    let create = requests[1].key.as_deref().unwrap();
    assert_ne!(upload, create);
    for (key, stage) in [(upload, "upload"), (create, "create")] {
        let prefix = format!("wiki-evidence-v1-{stage}-");
        assert!(key.starts_with(&prefix));
        let digest = &key[prefix.len()..];
        assert_eq!(
            digest,
            "69cd344d20fee04179a672ea3b2929da884e03975100369c926dedc642b5a364"
        );
        assert_eq!(digest.len(), 64);
        assert!(
            digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        );
        assert!(key.is_ascii() && key.len() <= 128);
    }
    assert_eq!(requests[0].key, requests[2].key);
    assert_eq!(requests[1].key, requests[3].key);
    assert_eq!(requests[1].body, requests[3].body);
    let unpadded = [
        "--idempotency-key",
        &parent,
        "evidence",
        "add-file",
        "--space",
        "CLI",
        "--title",
        "Evidence",
        "--file",
        file.to_str().unwrap(),
    ];
    let server2 = Server::start(vec![
        (201, json!({"id":"attachment"})),
        (201, json!({"id":"evidence"})),
    ])
    .await;
    success(&server2.run(&unpadded, None).await);
    assert_eq!(requests[0].key, server2.requests()[0].key);
    assert_eq!(requests[1].key, server2.requests()[1].key);
}

#[tokio::test]
async fn evidence_file_rejects_invalid_parent_before_upload() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("evidence.txt");
    std::fs::write(&file, b"evidence").unwrap();
    for key in [
        " ".to_owned(),
        "k".repeat(129),
        "ключ".to_owned(),
        "line\nbreak".to_owned(),
    ] {
        let server = Server::start(vec![
            (201, json!({"id":"attachment"})),
            (201, json!({"id":"evidence"})),
        ])
        .await;
        let output = server
            .run(
                &[
                    "--idempotency-key",
                    &key,
                    "--error-format",
                    "json",
                    "evidence",
                    "add-file",
                    "--title",
                    "Evidence",
                    "--file",
                    file.to_str().unwrap(),
                ],
                None,
            )
            .await;
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(serde_json::from_slice::<Value>(&output.stderr).unwrap()["error"].is_object());
        assert!(server.requests().is_empty(), "invalid key sent an upload");
    }
}
#[derive(Debug, Clone)]
struct Recorded {
    method: String,
    path: String,
    body: Vec<u8>,
    key: Option<String>,
    authorization: Option<String>,
}
struct TestState {
    responses: Mutex<VecDeque<(u16, Vec<u8>)>>,
    requests: Mutex<Vec<Recorded>>,
    delay: Duration,
}
struct Server {
    url: String,
    state: Arc<TestState>,
    handle: tokio::task::JoinHandle<()>,
}
impl Drop for Server {
    fn drop(&mut self) {
        self.handle.abort();
    }
}
impl Server {
    async fn start(responses: Vec<(u16, Value)>) -> Self {
        Self::bytes(
            responses
                .into_iter()
                .map(|(s, v)| (s, serde_json::to_vec(&v).unwrap()))
                .collect(),
            Duration::ZERO,
        )
        .await
    }
    async fn bytes(responses: Vec<(u16, Vec<u8>)>, delay: Duration) -> Self {
        let state = Arc::new(TestState {
            responses: Mutex::new(responses.into()),
            requests: Mutex::new(vec![]),
            delay,
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let app = Router::new().fallback(record).with_state(state.clone());
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Self { url, state, handle }
    }
    fn requests(&self) -> Vec<Recorded> {
        self.state.requests.lock().unwrap().clone()
    }
    async fn run(&self, args: &[&str], input: Option<&str>) -> std::process::Output {
        let url = format!("{}/api/v1", self.url);
        let args = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let input = input.map(str::to_string);
        tokio::task::spawn_blocking(move || {
            let mut child = Command::new(env!("CARGO_BIN_EXE_wiki"))
                .args(["--api-url", &url, "--token", "fixture-token"])
                .args(args)
                .env_remove("SDLC_API_TOKEN")
                .env_remove("CICD_PROFILE")
                .env_remove("WIKI_TOKEN")
                .env_remove("TASKTRACKER_TOKEN")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            if let Some(input) = input {
                child
                    .stdin
                    .take()
                    .unwrap()
                    .write_all(input.as_bytes())
                    .unwrap();
            } else {
                drop(child.stdin.take());
            }
            child.wait_with_output().unwrap()
        })
        .await
        .unwrap()
    }
}
async fn record(State(state): State<Arc<TestState>>, req: Request<Body>) -> Response {
    let method = req.method().to_string();
    let path = req.uri().to_string();
    let key = req
        .headers()
        .get("idempotency-key")
        .and_then(|h| h.to_str().ok())
        .map(str::to_string);
    let authorization = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .map(str::to_string);
    let body = to_bytes(req.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap()
        .to_vec();
    state.requests.lock().unwrap().push(Recorded {
        method,
        path,
        body,
        key,
        authorization,
    });
    tokio::time::sleep(state.delay).await;
    let (status, bytes) = state.responses.lock().unwrap().pop_front().unwrap_or((
        500,
        br#"{"error":{"code":"UNEXPECTED_REQUEST","message":"unexpected request"}}"#.to_vec(),
    ));
    Response::builder()
        .status(StatusCode::from_u16(status).unwrap())
        .header("content-type", "application/json")
        .body(Body::from(bytes))
        .unwrap()
}
fn success(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).unwrap()
}

#[tokio::test]
async fn external_idempotency_key_survives_process_retries() {
    let server = Server::start(vec![
        (200, json!({"id":"space"})),
        (200, json!({"id":"space"})),
    ])
    .await;
    for _ in 0..2 {
        success(
            &server
                .run(
                    &[
                        "--idempotency-key",
                        "stable-key",
                        "space",
                        "create",
                        "--key",
                        "CLI",
                        "--name",
                        "CLI space",
                    ],
                    None,
                )
                .await,
        );
    }
    let req = server.requests();
    assert_eq!(req[0].key.as_deref(), Some("stable-key"));
    assert_eq!(req[0].key, req[1].key);
    assert_eq!(req[0].body, req[1].body);
    assert_eq!(req[0].method, "POST");
    assert_eq!(req[0].path, "/api/v1/spaces");
    assert_eq!(
        req[0].authorization.as_deref(),
        Some("Bearer fixture-token")
    );
}
#[tokio::test]
async fn upload_and_download_do_not_clobber() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.txt");
    std::fs::write(&input, "upload fixture").unwrap();
    let out = dir.path().join("output.txt");
    let server = Server::bytes(
        vec![
            (201, br#"{"id":"attachment"}"#.to_vec()),
            (200, b"downloaded bytes".to_vec()),
        ],
        Duration::ZERO,
    )
    .await;
    success(
        &server
            .run(
                &["attachment", "upload", "--file", input.to_str().unwrap()],
                None,
            )
            .await,
    );
    success(
        &server
            .run(
                &[
                    "attachment",
                    "download",
                    "attachment",
                    "--out",
                    out.to_str().unwrap(),
                ],
                None,
            )
            .await,
    );
    let failed = server
        .run(
            &[
                "attachment",
                "download",
                "attachment",
                "--out",
                out.to_str().unwrap(),
            ],
            None,
        )
        .await;
    assert!(!failed.status.success());
    assert_eq!(server.requests().len(), 2);
    assert_eq!(std::fs::read(&out).unwrap(), b"downloaded bytes");
    assert!(String::from_utf8_lossy(&server.requests()[0].body).contains("upload fixture"));
}
#[tokio::test]
async fn structured_errors_timeout_and_bad_usage() {
    for status in [401, 403, 409, 429, 500] {
        let server=Server::start(vec![(status,json!({"error":{"code":"FIXTURE_ERROR","message":"fixture-token","request_id":"req-fixture"}}))]).await;
        let failed = server
            .run(&["--error-format", "json", "space", "list"], None)
            .await;
        assert_eq!(failed.status.code(), Some(1));
        assert!(failed.stdout.is_empty());
        assert_eq!(
            serde_json::from_slice::<Value>(&failed.stderr).unwrap()["error"]["status"],
            status
        );
        assert!(!String::from_utf8_lossy(&failed.stderr).contains("fixture-token"));
    }
    let server = Server::bytes(vec![(200, b"{}".to_vec())], Duration::from_secs(3)).await;
    let start = std::time::Instant::now();
    let failed = server
        .run(&["--timeout-seconds", "1", "space", "list"], None)
        .await;
    assert_eq!(failed.status.code(), Some(1));
    assert!(start.elapsed() < Duration::from_secs(2));
    let failed = server
        .run(&["--timeout-seconds", "0", "space", "list"], None)
        .await;
    assert_eq!(failed.status.code(), Some(2));
}

#[tokio::test]
async fn parsing_errors_are_json_and_do_not_echo_credentials() {
    let server = Server::start(vec![]).await;
    let output = server
        .run(
            &[
                "--error-format",
                "json",
                "--token",
                "sensitive-value",
                "unknown-command",
            ],
            None,
        )
        .await;
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let value: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(value["error"]["code"], "CLI_USAGE");
    assert!(!String::from_utf8_lossy(&output.stderr).contains("sensitive-value"));
    assert!(server.requests().is_empty());
}

#[tokio::test]
async fn token_priority_is_flag_then_wiki_then_shared() {
    let server = Server::start(vec![(200, json!([])), (200, json!([])), (200, json!([]))]).await;
    for choice in 0..3 {
        let url = format!("{}/api/v1", server.url);
        let output = tokio::task::spawn_blocking(move || {
            let mut command = Command::new(env!("CARGO_BIN_EXE_wiki"));
            command
                .env("WIKI_TOKEN", "wiki-token")
                .env("SDLC_API_TOKEN", "common-token")
                .args(["--api-url", &url]);
            if choice == 0 {
                command.args(["--token", "flag-token"]);
            }
            if choice == 2 {
                command.env_remove("WIKI_TOKEN");
            }
            command.args(["space", "list"]).output().unwrap()
        })
        .await
        .unwrap();
        success(&output);
    }
    let requests = server.requests();
    for (request, token) in requests
        .iter()
        .zip(["flag-token", "wiki-token", "common-token"])
    {
        assert_eq!(
            request.authorization.as_deref(),
            Some(format!("Bearer {token}").as_str())
        );
    }
}

#[tokio::test]
async fn trimmed_credential_in_api_error_is_redacted() {
    let server = Server::start(vec![(
        403,
        json!({"error": {"code": "DENIED", "message": "fixture-token"}}),
    )])
    .await;
    let url = format!("{}/api/v1", server.url);
    let output = tokio::task::spawn_blocking(move || {
        Command::new(env!("CARGO_BIN_EXE_wiki"))
            .args([
                "--api-url",
                &url,
                "--token",
                " fixture-token ",
                "--error-format",
                "json",
                "space",
                "list",
            ])
            .env_remove("WIKI_TOKEN")
            .env_remove("SDLC_API_TOKEN")
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["status"], 403);
    assert_eq!(error["error"]["message"], "[REDACTED]");
}

#[test]
fn help_hides_token_environment_value() {
    let output = Command::new(env!("CARGO_BIN_EXE_wiki"))
        .args(["--help"])
        .env("WIKI_TOKEN", "help-only-fixture-token")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("WIKI_TOKEN"));
    assert!(!stdout.contains("help-only-fixture-token"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("help-only-fixture-token"));
}
