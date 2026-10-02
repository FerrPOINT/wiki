use std::sync::Arc;
fn test_config_with_registration(registration_enabled: bool) -> Arc<shared::AppConfig> {
    Arc::new(shared::AppConfig {
        environment: shared::RuntimeEnvironment::Test,
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
        maintenance: shared::MaintenanceConfig::default(),
        email: shared::EmailConfig::default(),
        bootstrap: shared::BootstrapConfig::default(),
    })
}

fn run_output(url: &str, token: &str, args: &[&str], input: Option<&str>) -> std::process::Output {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new(env!("CARGO_BIN_EXE_wiki"))
        .env_remove("SDLC_API_TOKEN")
        .env_remove("WIKI_TOKEN")
        .env_remove("TASK_TRACKER_TOKEN")
        .args([
            "--api-url",
            &format!("{url}/api/v1"),
            "--token",
            token,
            "--output",
            "json",
        ])
        .args(args)
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
    }
    child.wait_with_output().unwrap()
}

fn run(url: &str, token: &str, args: &[&str], input: Option<&str>) -> serde_json::Value {
    let output = run_output(url, token, args, input);
    assert!(
        output.status.success(),
        "{:?}: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wiki_document_lifecycle_and_replay_against_real_api() {
    let ctx = Arc::new(app::WikiAppContext::new(test_config_with_registration(
        true,
    )));
    let router = api::router_for_memory_tests(ctx.clone()).with_state(ctx);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let auth: serde_json::Value = reqwest::Client::new()
        .post(format!("{url}/api/v1/auth/login"))
        .json(&serde_json::json!({"email":"demo@example.com","password":"demo"}))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = auth["access_token"].as_str().unwrap();
    run(
        &url,
        token,
        &["space", "create", "--key", "CLI", "--name", "CLI fixture"],
        None,
    );
    run(&url, token, &["space", "tree", "CLI"], None);
    let template = run(
        &url,
        token,
        &[
            "template",
            "create",
            "--name",
            "CLI template",
            "--type",
            "release_note",
            "--from-file",
            "-",
        ],
        Some("# Template"),
    );
    run(
        &url,
        token,
        &[
            "template",
            "apply",
            template["id"].as_str().unwrap(),
            "--space",
            "CLI",
            "--title",
            "Template document",
        ],
        None,
    );
    let args = [
        "--idempotency-key",
        "cli-document-replay",
        "doc",
        "create",
        "--space",
        "CLI",
        "--title",
        "CLI doc",
        "--task",
        "PROJ-1",
        "--phase",
        "implementation",
        "--from-file",
        "-",
    ];
    let doc = run(&url, token, &args, Some("# First"));
    assert_eq!(run(&url, token, &args, Some("# First"))["id"], doc["id"]);
    let id = doc["id"].as_str().unwrap();
    for operation in ["get", "docs", "evidence"] {
        run(
            &url,
            token,
            &["task", operation, "--space", "CLI", "--key", "PROJ-1"],
            None,
        );
        run(
            &url,
            token,
            &[
                "phase",
                operation,
                "--space",
                "CLI",
                "--key",
                "implementation",
            ],
            None,
        );
    }
    run(
        &url,
        token,
        &["doc", "draft", id, "--from-file", "-"],
        Some("# Published fixture"),
    );
    run(&url, token, &["doc", "publish", id], None);
    let history = run(
        &url,
        token,
        &["doc", "history", id, "--limit", "10", "--offset", "0"],
        None,
    );
    let rev = history["revisions"][0]["id"].as_str().unwrap();
    run(&url, token, &["doc", "revision", id, rev], None);
    run(&url, token, &["doc", "move", id], None);
    run(
        &url,
        token,
        &[
            "evidence",
            "add-link",
            "--space",
            "CLI",
            "--document",
            id,
            "--title",
            "Evidence",
            "--url",
            "https://example.invalid/fixture",
        ],
        None,
    );
    run(
        &url,
        token,
        &["evidence", "list", "--space", "CLI", "--limit", "10"],
        None,
    );
    run(
        &url,
        token,
        &["task", "list", "--space", "CLI", "--limit", "10"],
        None,
    );
    run(
        &url,
        token,
        &["phase", "list", "--space", "CLI", "--limit", "10"],
        None,
    );
    run(
        &url,
        token,
        &[
            "search", "query", "fixture", "--space", "CLI", "--limit", "10",
        ],
        None,
    );
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("sample.txt");
    std::fs::write(&file, b"wiki file").unwrap();
    let evidence_key = "e".repeat(128);
    let evidence_args = [
        "--idempotency-key",
        &evidence_key,
        "--error-format",
        "json",
        "evidence",
        "add-file",
        "--space",
        "CLI",
        "--document",
        id,
        "--title",
        "File evidence",
        "--file",
        file.to_str().unwrap(),
    ];
    let before = run(
        &url,
        token,
        &["evidence", "list", "--space", "CLI", "--limit", "100"],
        None,
    );
    let evidence = run(&url, token, &evidence_args, None);
    let repeated = run(&url, token, &evidence_args, None);
    assert_eq!(evidence["id"], repeated["id"]);
    assert_eq!(evidence["attachment_id"], repeated["attachment_id"]);
    let after = run(
        &url,
        token,
        &["evidence", "list", "--space", "CLI", "--limit", "100"],
        None,
    );
    assert_eq!(
        after["evidence"].as_array().unwrap().len(),
        before["evidence"].as_array().unwrap().len() + 1
    );
    let evidence_download = dir.path().join("evidence-download.txt");
    run(
        &url,
        token,
        &[
            "attachment",
            "download",
            evidence["attachment_id"].as_str().unwrap(),
            "--out",
            evidence_download.to_str().unwrap(),
        ],
        None,
    );
    assert_eq!(std::fs::read(&evidence_download).unwrap(), b"wiki file");
    let mut changed_title = evidence_args;
    changed_title[11] = "Changed title";
    let conflict = run_output(&url, token, &changed_title, None);
    assert_eq!(conflict.status.code(), Some(1));
    assert!(conflict.stdout.is_empty());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&conflict.stderr).unwrap()["error"]["status"],
        409
    );
    std::fs::write(&file, b"changed evidence").unwrap();
    let conflict = run_output(&url, token, &evidence_args, None);
    assert_eq!(conflict.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&conflict.stderr).unwrap()["error"]["status"],
        409
    );
    std::fs::write(&file, b"wiki file").unwrap();
    let attachment = run(
        &url,
        token,
        &[
            "--idempotency-key",
            "cli-upload-replay",
            "attachment",
            "upload",
            "--file",
            file.to_str().unwrap(),
        ],
        None,
    );
    let out = dir.path().join("download.txt");
    let replay = run(
        &url,
        token,
        &[
            "--idempotency-key",
            "cli-upload-replay",
            "attachment",
            "upload",
            "--file",
            file.to_str().unwrap(),
        ],
        None,
    );
    assert_eq!(replay["id"], attachment["id"]);
    run(
        &url,
        token,
        &[
            "attachment",
            "download",
            attachment["id"].as_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ],
        None,
    );
    assert_eq!(std::fs::read(out).unwrap(), b"wiki file");
    std::fs::write(&file, b"changed content").unwrap();
    let conflict = std::process::Command::new(env!("CARGO_BIN_EXE_wiki"))
        .args([
            "--api-url",
            &format!("{url}/api/v1"),
            "--token",
            token,
            "--error-format",
            "json",
            "--idempotency-key",
            "cli-upload-replay",
            "attachment",
            "upload",
            "--file",
            file.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(conflict.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&conflict.stderr).unwrap()["error"]["status"],
        409
    );
    run(&url, token, &["doc", "archive", id], None);
    server.abort();
}
