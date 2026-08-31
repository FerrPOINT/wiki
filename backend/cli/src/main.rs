use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use reqwest::{Client, Method, multipart};
use serde_json::{Value, json};
use std::{
    io::Read,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Parser)]
#[command(name = "wiki")]
#[command(about = "Wiki CLI -- HTTP client for spaces, documents, task/phase links and evidence")]
struct Cli {
    #[arg(
        long,
        env = "WIKI_API_URL",
        default_value = "http://localhost:3456/api/v1"
    )]
    api_url: String,

    #[arg(long, env = "WIKI_TOKEN")]
    token: Option<String>,

    #[arg(long, env = "WIKI_OUTPUT", value_enum, default_value = "json")]
    output: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Json,
    Table,
    Compact,
}

#[derive(Subcommand)]
enum Commands {
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    Space {
        #[command(subcommand)]
        command: SpaceCommands,
    },
    Doc {
        #[command(subcommand)]
        command: DocCommands,
    },
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
    Phase {
        #[command(subcommand)]
        command: PhaseCommands,
    },
    Evidence {
        #[command(subcommand)]
        command: EvidenceCommands,
    },
    Template {
        #[command(subcommand)]
        command: TemplateCommands,
    },
    Search {
        #[command(subcommand)]
        command: SearchCommands,
    },
    Settings {
        #[command(subcommand)]
        command: SettingsCommands,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    Login {
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    Logout,
    Whoami,
}

#[derive(Subcommand)]
enum SpaceCommands {
    List,
    Create {
        #[arg(long)]
        key: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        description: Option<String>,
    },
    Get {
        key: String,
    },
    Tree {
        key: String,
    },
    Members {
        key: String,
    },
}

#[derive(Subcommand)]
enum DocCommands {
    Create(DocCreateArgs),
    Get {
        document_id: String,
    },
    Draft(DocContentArgs),
    Publish {
        document_id: String,
        #[arg(long)]
        summary: Option<String>,
    },
    Archive {
        document_id: String,
    },
    Move {
        document_id: String,
        #[arg(long)]
        parent: Option<String>,
    },
    History {
        document_id: String,
    },
}

#[derive(Args)]
struct DocCreateArgs {
    #[arg(long)]
    space: String,
    #[arg(long)]
    title: String,
    #[arg(long = "type", default_value = "page")]
    document_type: String,
    #[arg(long)]
    parent_id: Option<String>,
    #[arg(long)]
    slug: Option<String>,
    #[arg(long)]
    task: Option<String>,
    #[arg(long)]
    phase: Option<String>,
    #[arg(long)]
    from_file: PathBuf,
}

#[derive(Args)]
struct DocContentArgs {
    document_id: String,
    #[arg(long)]
    from_file: PathBuf,
}

#[derive(Subcommand)]
enum TaskCommands {
    Get(LinkTargetArgs),
    Docs(LinkTargetArgs),
    Evidence(LinkTargetArgs),
    LinkDoc(LinkDocumentArgs),
}

#[derive(Subcommand)]
enum PhaseCommands {
    Get(LinkTargetArgs),
    Docs(LinkTargetArgs),
    Evidence(LinkTargetArgs),
    LinkDoc(LinkDocumentArgs),
}

#[derive(Args)]
struct LinkTargetArgs {
    #[arg(long)]
    space: String,
    #[arg(long)]
    key: String,
}

#[derive(Args)]
struct LinkDocumentArgs {
    #[arg(long)]
    space: String,
    #[arg(long)]
    key: String,
    #[arg(long)]
    document: String,
}

#[derive(Subcommand)]
enum EvidenceCommands {
    AddLink(EvidenceLinkArgs),
    AddFile(EvidenceFileArgs),
    Get {
        evidence_id: String,
    },
    List {
        #[arg(long)]
        space: Option<String>,
        #[arg(long)]
        document: Option<String>,
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        phase: Option<String>,
    },
}

#[derive(Args)]
struct EvidenceLinkArgs {
    #[arg(long)]
    space: Option<String>,
    #[arg(long)]
    document: Option<String>,
    #[arg(long)]
    task: Option<String>,
    #[arg(long)]
    phase: Option<String>,
    #[arg(long = "type", default_value = "external_url")]
    evidence_type: String,
    #[arg(long)]
    title: String,
    #[arg(long)]
    url: String,
}

#[derive(Args)]
struct EvidenceFileArgs {
    #[arg(long)]
    space: Option<String>,
    #[arg(long)]
    document: Option<String>,
    #[arg(long)]
    task: Option<String>,
    #[arg(long)]
    phase: Option<String>,
    #[arg(long = "type", default_value = "uploaded_file")]
    evidence_type: String,
    #[arg(long)]
    title: String,
    #[arg(long)]
    file: PathBuf,
}

#[derive(Subcommand)]
enum TemplateCommands {
    List,
    Apply(TemplateApplyArgs),
}

#[derive(Args)]
struct TemplateApplyArgs {
    template: String,
    #[arg(long)]
    space: String,
    #[arg(long)]
    title: String,
    #[arg(long)]
    parent_id: Option<String>,
    #[arg(long)]
    task: Option<String>,
    #[arg(long)]
    phase: Option<String>,
}

#[derive(Subcommand)]
enum SearchCommands {
    Query {
        query: String,
        #[arg(long)]
        space: Option<String>,
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        phase: Option<String>,
        #[arg(long = "type")]
        document_type: Option<String>,
    },
}

#[derive(Subcommand)]
enum SettingsCommands {
    Get,
}

struct ApiClient {
    base_url: String,
    token: Option<String>,
    client: Client,
}

impl ApiClient {
    fn new(base_url: String, token: Option<String>) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
            client: Client::new(),
        }
    }

    async fn get(&self, path: &str) -> Result<Value> {
        self.request(Method::GET, path).send_json().await
    }

    async fn post_json(&self, path: &str, body: Value) -> Result<Value> {
        self.write_json(Method::POST, path, body).await
    }

    async fn put_json(&self, path: &str, body: Value) -> Result<Value> {
        self.write_json(Method::PUT, path, body).await
    }

    async fn post_multipart(&self, path: &str, form: multipart::Form) -> Result<Value> {
        self.request(Method::POST, path)
            .header("Idempotency-Key", idempotency_key("upload"))
            .multipart(form)
            .send_json()
            .await
    }

    async fn write_json(&self, method: Method, path: &str, body: Value) -> Result<Value> {
        self.request(method, path)
            .header("Idempotency-Key", idempotency_key("write"))
            .json(&body)
            .send_json()
            .await
    }

    fn request(&self, method: Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let request = self.client.request(method, url);
        match &self.token {
            Some(token) if !token.trim().is_empty() => request.bearer_auth(token),
            _ => request,
        }
    }
}

trait SendJson {
    async fn send_json(self) -> Result<Value>;
}

impl SendJson for reqwest::RequestBuilder {
    async fn send_json(self) -> Result<Value> {
        let response = self.send().await.context("request failed")?;
        let status = response.status();
        let text = response.text().await.context("failed to read response")?;
        if !status.is_success() {
            bail!("{}", format_api_error(status, &text));
        }
        if text.trim().is_empty() {
            return Ok(json!({ "status": "ok" }));
        }
        serde_json::from_str(&text).context("API returned non-JSON response")
    }
}

fn format_api_error(status: reqwest::StatusCode, body: &str) -> String {
    let body = body.trim();
    if body.is_empty() {
        return format!("API returned {status}");
    }

    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return format!("API returned {status}: {body}");
    };
    let Some(error) = value.get("error") else {
        return format!("API returned {status}: {body}");
    };

    let message = match error {
        Value::String(message) => message.clone(),
        Value::Object(map) => {
            let code = map.get("code").and_then(Value::as_str);
            let message = map.get("message").and_then(Value::as_str);
            let mut parts = Vec::new();

            match (code, message) {
                (Some(code), Some(message)) => parts.push(format!("{code}: {message}")),
                (Some(code), None) => parts.push(code.to_string()),
                (None, Some(message)) => parts.push(message.to_string()),
                (None, None) => {}
            }

            if let Some(request_id) = map
                .get("requestId")
                .or_else(|| map.get("request_id"))
                .and_then(Value::as_str)
            {
                parts.push(format!("requestId={request_id}"));
            }
            if let Some(details) = format_api_error_details(map.get("details")) {
                parts.push(format!("details={details}"));
            }

            if parts.is_empty() {
                error.to_string()
            } else {
                parts.join("; ")
            }
        }
        other => other.to_string(),
    };

    format!("API returned {status}: {message}")
}

fn format_api_error_details(details: Option<&Value>) -> Option<String> {
    let details = details?.as_array()?;
    let parts = details
        .iter()
        .filter_map(|detail| {
            let field = detail.get("field").and_then(Value::as_str);
            let message = detail.get("message").and_then(Value::as_str);
            match (field, message) {
                (Some(field), Some(message)) => Some(format!("{field}: {message}")),
                (None, Some(message)) => Some(message.to_string()),
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let api = ApiClient::new(cli.api_url, cli.token);
    let value = execute(&api, cli.command).await?;
    print_value(&value, cli.output)?;
    Ok(())
}

async fn execute(api: &ApiClient, command: Commands) -> Result<Value> {
    match command {
        Commands::Auth { command } => execute_auth(api, command).await,
        Commands::Space { command } => execute_space(api, command).await,
        Commands::Doc { command } => execute_doc(api, command).await,
        Commands::Task { command } => execute_task(api, command).await,
        Commands::Phase { command } => execute_phase(api, command).await,
        Commands::Evidence { command } => execute_evidence(api, command).await,
        Commands::Template { command } => execute_template(api, command).await,
        Commands::Search { command } => execute_search(api, command).await,
        Commands::Settings { command } => execute_settings(api, command).await,
    }
}

async fn execute_auth(api: &ApiClient, command: AuthCommands) -> Result<Value> {
    match command {
        AuthCommands::Login { email, password } => {
            api.post_json(
                "/auth/login",
                json!({ "email": email, "password": password }),
            )
            .await
        }
        AuthCommands::Logout => api.post_json("/auth/logout", json!({})).await,
        AuthCommands::Whoami => api.get("/users/me").await,
    }
}

async fn execute_space(api: &ApiClient, command: SpaceCommands) -> Result<Value> {
    match command {
        SpaceCommands::List => api.get("/spaces").await,
        SpaceCommands::Create {
            key,
            name,
            description,
        } => {
            api.post_json(
                "/spaces",
                json!({ "key": key, "name": name, "description": description }),
            )
            .await
        }
        SpaceCommands::Get { key } => api.get(&format!("/spaces/{}", enc(&key))).await,
        SpaceCommands::Tree { key } => api.get(&format!("/spaces/{}/tree", enc(&key))).await,
        SpaceCommands::Members { key } => api.get(&format!("/spaces/{}/members", enc(&key))).await,
    }
}

async fn execute_doc(api: &ApiClient, command: DocCommands) -> Result<Value> {
    match command {
        DocCommands::Create(args) => {
            let content = read_markdown(&args.from_file)?;
            api.post_json(
                &format!("/spaces/{}/documents", enc(&args.space)),
                json!({
                    "title": args.title,
                    "document_type": args.document_type,
                    "parent_id": args.parent_id,
                    "slug": args.slug,
                    "task_key": args.task,
                    "phase_key": args.phase,
                    "content_markdown": content
                }),
            )
            .await
        }
        DocCommands::Get { document_id } => {
            api.get(&format!("/documents/{}", enc(&document_id))).await
        }
        DocCommands::Draft(args) => {
            let content = read_markdown(&args.from_file)?;
            api.put_json(
                &format!("/documents/{}/draft", enc(&args.document_id)),
                json!({ "content_markdown": content }),
            )
            .await
        }
        DocCommands::Publish {
            document_id,
            summary,
        } => {
            api.post_json(
                &format!("/documents/{}/publish", enc(&document_id)),
                json!({ "summary": summary }),
            )
            .await
        }
        DocCommands::Archive { document_id } => {
            api.post_json(
                &format!("/documents/{}/archive", enc(&document_id)),
                json!({}),
            )
            .await
        }
        DocCommands::Move {
            document_id,
            parent,
        } => {
            api.post_json(
                &format!("/documents/{}/move", enc(&document_id)),
                json!({ "parent_id": parent }),
            )
            .await
        }
        DocCommands::History { document_id } => {
            api.get(&format!("/documents/{}/revisions", enc(&document_id)))
                .await
        }
    }
}

async fn execute_task(api: &ApiClient, command: TaskCommands) -> Result<Value> {
    match command {
        TaskCommands::Get(args) => api.get(&task_path(&args.space, &args.key)).await,
        TaskCommands::Docs(args) => {
            api.get(&format!("{}/documents", task_path(&args.space, &args.key)))
                .await
        }
        TaskCommands::Evidence(args) => {
            api.get(&format!("{}/evidence", task_path(&args.space, &args.key)))
                .await
        }
        TaskCommands::LinkDoc(args) => {
            api.post_json(
                &format!("{}/links/documents", task_path(&args.space, &args.key)),
                json!({ "document_id": args.document }),
            )
            .await
        }
    }
}

async fn execute_phase(api: &ApiClient, command: PhaseCommands) -> Result<Value> {
    match command {
        PhaseCommands::Get(args) => api.get(&phase_path(&args.space, &args.key)).await,
        PhaseCommands::Docs(args) => {
            api.get(&format!("{}/documents", phase_path(&args.space, &args.key)))
                .await
        }
        PhaseCommands::Evidence(args) => {
            api.get(&format!("{}/evidence", phase_path(&args.space, &args.key)))
                .await
        }
        PhaseCommands::LinkDoc(args) => {
            api.post_json(
                &format!("{}/links/documents", phase_path(&args.space, &args.key)),
                json!({ "document_id": args.document }),
            )
            .await
        }
    }
}

async fn execute_evidence(api: &ApiClient, command: EvidenceCommands) -> Result<Value> {
    match command {
        EvidenceCommands::AddLink(args) => {
            api.post_json(
                "/evidence",
                json!({
                    "space": args.space,
                    "document_id": args.document,
                    "task_key": args.task,
                    "phase_key": args.phase,
                    "evidence_type": args.evidence_type,
                    "title": args.title,
                    "url": args.url
                }),
            )
            .await
        }
        EvidenceCommands::AddFile(args) => {
            let bytes = std::fs::read(&args.file)
                .with_context(|| format!("failed to read {}", args.file.display()))?;
            let filename = args
                .file
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "attachment".to_string());
            let form = multipart::Form::new().part(
                "file",
                multipart::Part::bytes(bytes).file_name(filename.clone()),
            );
            let attachment = api.post_multipart("/attachments", form).await?;
            api.post_json(
                "/evidence",
                json!({
                    "space": args.space,
                    "document_id": args.document,
                    "task_key": args.task,
                    "phase_key": args.phase,
                    "evidence_type": args.evidence_type,
                    "title": args.title,
                    "attachment_id": attachment.get("id").cloned().unwrap_or(Value::Null),
                    "checksum": attachment.get("checksum").cloned().unwrap_or(Value::Null)
                }),
            )
            .await
        }
        EvidenceCommands::Get { evidence_id } => {
            api.get(&format!("/evidence/{}", enc(&evidence_id))).await
        }
        EvidenceCommands::List {
            space,
            document,
            task,
            phase,
        } => {
            let query = query_string([
                ("space", space),
                ("document_id", document),
                ("task_key", task),
                ("phase_key", phase),
            ]);
            api.get(&format!("/evidence{}", query)).await
        }
    }
}

async fn execute_template(api: &ApiClient, command: TemplateCommands) -> Result<Value> {
    match command {
        TemplateCommands::List => api.get("/templates").await,
        TemplateCommands::Apply(args) => {
            let templates = api.get("/templates").await?;
            let template = find_template(&templates, &args.template)?;
            api.post_json(
                &format!("/spaces/{}/documents", enc(&args.space)),
                json!({
                    "title": args.title,
                    "document_type": template
                        .get("document_type")
                        .cloned()
                        .unwrap_or_else(|| json!("page")),
                    "parent_id": args.parent_id,
                    "task_key": args.task,
                    "phase_key": args.phase,
                    "content_markdown": template
                        .get("body_markdown")
                        .cloned()
                        .unwrap_or_else(|| json!(""))
                }),
            )
            .await
        }
    }
}

async fn execute_search(api: &ApiClient, command: SearchCommands) -> Result<Value> {
    match command {
        SearchCommands::Query {
            query,
            space,
            task,
            phase,
            document_type,
        } => {
            let query = query_string([
                ("q", Some(query)),
                ("space", space),
                ("task_key", task),
                ("phase_key", phase),
                ("document_type", document_type),
            ]);
            api.get(&format!("/search{}", query)).await
        }
    }
}

async fn execute_settings(api: &ApiClient, command: SettingsCommands) -> Result<Value> {
    match command {
        SettingsCommands::Get => api.get("/settings").await,
    }
}

fn find_template<'a>(value: &'a Value, requested: &str) -> Result<&'a Value> {
    let templates = value
        .get("templates")
        .and_then(Value::as_array)
        .context("API response does not contain templates array")?;
    let requested_lower = requested.to_ascii_lowercase();
    templates
        .iter()
        .find(|template| {
            template
                .get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| id.eq_ignore_ascii_case(requested))
                || template
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| name.to_ascii_lowercase() == requested_lower)
                || template
                    .get("document_type")
                    .and_then(Value::as_str)
                    .is_some_and(|document_type| document_type.eq_ignore_ascii_case(requested))
        })
        .with_context(|| format!("template {requested} not found"))
}

fn read_markdown(path: &PathBuf) -> Result<String> {
    if path.as_os_str() == "-" {
        let mut input = String::new();
        std::io::stdin()
            .read_to_string(&mut input)
            .context("failed to read stdin")?;
        return Ok(input);
    }
    std::fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
}

fn print_value(value: &Value, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(value)?);
        }
        OutputFormat::Table => print_table(value),
        OutputFormat::Compact => print_compact(value),
    }
    Ok(())
}

fn print_table(value: &Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                println!("{}", compact_line(item));
            }
        }
        Value::Object(map) => {
            if let Some(items) = map.values().find_map(Value::as_array) {
                for item in items {
                    println!("{}", compact_line(item));
                }
            } else {
                println!("{}", compact_line(value));
            }
        }
        other => println!("{}", compact_line(other)),
    }
}

fn print_compact(value: &Value) {
    println!("{}", compact_line(value));
}

fn compact_line(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let preferred = [
                "id",
                "key",
                "document_id",
                "task_key",
                "phase_key",
                "title",
                "name",
                "status",
                "document_type",
                "url",
            ];
            let fields: Vec<String> = preferred
                .iter()
                .filter_map(|key| map.get(*key).map(|value| scalar_to_string(key, value)))
                .collect();
            if fields.is_empty() {
                value.to_string()
            } else {
                fields.join(" | ")
            }
        }
        other => other.to_string(),
    }
}

fn scalar_to_string(key: &str, value: &Value) -> String {
    match value {
        Value::String(text) => format!("{key}={text}"),
        other => format!("{key}={other}"),
    }
}

fn task_path(space: &str, key: &str) -> String {
    format!("/spaces/{}/tasks/{}", enc(space), enc(key))
}

fn phase_path(space: &str, key: &str) -> String {
    format!("/spaces/{}/phases/{}", enc(space), enc(key))
}

fn query_string<const N: usize>(pairs: [(&str, Option<String>); N]) -> String {
    let parts: Vec<String> = pairs
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| format!("{}={}", enc(key), enc(&value))))
        .collect();
    if parts.is_empty() {
        String::new()
    } else {
        format!("?{}", parts.join("&"))
    }
}

fn enc(value: &str) -> String {
    urlencoding::encode(value).into_owned()
}

fn idempotency_key(scope: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("wiki-cli-{scope}-{nanos}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Json, Router,
        body::{Body, to_bytes},
        extract::State,
        http::{HeaderMap, Request, StatusCode},
        response::{IntoResponse, Response},
    };
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;

    #[derive(Clone, Debug)]
    struct RecordedRequest {
        method: Method,
        path: String,
        authorization: Option<String>,
        content_type: Option<String>,
        idempotency_key: Option<String>,
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

    async fn record_request(
        State(state): State<Arc<MockState>>,
        request: Request<Body>,
    ) -> Response {
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
            body: body.to_vec(),
        });

        let (status, payload) = match path_only.as_str() {
            "/api/v1/error-object" => (
                StatusCode::BAD_REQUEST,
                json!({
                    "error": {
                        "code": "VALIDATION_ERROR",
                        "message": "Request validation failed",
                        "requestId": "req-1",
                        "details": [{ "field": "summary", "message": "required" }]
                    }
                }),
            ),
            "/api/v1/error-string" => (
                StatusCode::NOT_FOUND,
                json!({ "error": "document product-requirements not found" }),
            ),
            "/api/v1/attachments" => (
                StatusCode::OK,
                json!({ "id": "attachment-1", "checksum": "sha256-test" }),
            ),
            "/api/v1/templates" => (
                StatusCode::OK,
                json!({
                    "templates": [{
                        "id": "00000000-0000-0000-0000-000000000042",
                        "name": "Требования",
                        "document_type": "requirements",
                        "body_markdown": "# Requirements\n\nTemplate body"
                    }]
                }),
            ),
            "/api/v1/settings" => (
                StatusCode::OK,
                json!({
                    "instance_name": "Wiki",
                    "api_base_path": "/api/v1",
                    "default_space_key": "SDLC",
                    "default_language": "ru",
                    "timezone": "Europe/Moscow",
                    "registration_enabled": true,
                    "public_links_enabled": false,
                    "search_backend": "PostgreSQL FTS",
                    "storage_backend": "local",
                    "max_upload_bytes": 26214400,
                    "markdown_renderer": "comrak",
                    "html_sanitizer": "ammonia"
                }),
            ),
            _ => (StatusCode::OK, json!({ "status": "ok" })),
        };

        (status, Json(payload)).into_response()
    }

    fn header_string(headers: &HeaderMap, key: &str) -> Option<String> {
        headers
            .get(key)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned)
    }

    fn assert_write_idempotency_key(request: &RecordedRequest) {
        assert!(
            request
                .idempotency_key
                .as_deref()
                .is_some_and(|value| value.starts_with("wiki-cli-write-"))
        );
    }

    #[tokio::test]
    async fn api_error_envelopes_are_rendered_as_cli_errors() {
        let server = spawn_mock_server().await;
        let api = ApiClient::new(server.api_url.clone(), Some("secret-token".to_string()));

        let structured = api.get("/error-object").await.unwrap_err().to_string();
        assert_eq!(
            structured,
            "API returned 400 Bad Request: VALIDATION_ERROR: Request validation failed; requestId=req-1; details=summary: required"
        );

        let simple = api.get("/error-string").await.unwrap_err().to_string();
        assert_eq!(
            simple,
            "API returned 404 Not Found: document product-requirements not found"
        );
    }

    #[tokio::test]
    async fn search_query_builds_filtered_get_request() {
        let server = spawn_mock_server().await;
        let api = ApiClient::new(server.api_url.clone(), Some("secret-token".to_string()));

        let value = execute(
            &api,
            Commands::Search {
                command: SearchCommands::Query {
                    query: "release gate".to_string(),
                    space: Some("SDLC KB".to_string()),
                    task: Some("SDLC-42".to_string()),
                    phase: Some("testing".to_string()),
                    document_type: Some("requirements".to_string()),
                },
            },
        )
        .await
        .unwrap();

        assert_eq!(value["status"], "ok");
        let requests = server.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, Method::GET);
        assert_eq!(
            requests[0].path,
            "/api/v1/search?q=release%20gate&space=SDLC%20KB&task_key=SDLC-42&phase_key=testing&document_type=requirements"
        );
        assert_eq!(
            requests[0].authorization.as_deref(),
            Some("Bearer secret-token")
        );
        assert!(requests[0].idempotency_key.is_none());
    }

    #[tokio::test]
    async fn auth_commands_use_public_api_paths_and_auth_headers() {
        let server = spawn_mock_server().await;
        let anonymous_api = ApiClient::new(server.api_url.clone(), None);
        let authed_api = ApiClient::new(server.api_url.clone(), Some("secret-token".to_string()));

        let login = execute(
            &anonymous_api,
            Commands::Auth {
                command: AuthCommands::Login {
                    email: "editor@example.com".to_string(),
                    password: "secret".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let logout = execute(
            &authed_api,
            Commands::Auth {
                command: AuthCommands::Logout,
            },
        )
        .await
        .unwrap();
        let whoami = execute(
            &authed_api,
            Commands::Auth {
                command: AuthCommands::Whoami,
            },
        )
        .await
        .unwrap();

        for value in [login, logout, whoami] {
            assert_eq!(value["status"], "ok");
        }

        let requests = server.requests();
        assert_eq!(requests.len(), 3);

        let login = &requests[0];
        assert_eq!(login.method, Method::POST);
        assert_eq!(login.path, "/api/v1/auth/login");
        assert!(login.authorization.is_none());
        assert_write_idempotency_key(login);
        let body: Value = serde_json::from_slice(&login.body).unwrap();
        assert_eq!(body["email"], "editor@example.com");
        assert_eq!(body["password"], "secret");

        let logout = &requests[1];
        assert_eq!(logout.method, Method::POST);
        assert_eq!(logout.path, "/api/v1/auth/logout");
        assert_eq!(logout.authorization.as_deref(), Some("Bearer secret-token"));
        assert_write_idempotency_key(logout);

        let whoami = &requests[2];
        assert_eq!(whoami.method, Method::GET);
        assert_eq!(whoami.path, "/api/v1/users/me");
        assert_eq!(whoami.authorization.as_deref(), Some("Bearer secret-token"));
        assert!(whoami.idempotency_key.is_none());
    }

    #[tokio::test]
    async fn space_commands_use_public_api_paths() {
        let server = spawn_mock_server().await;
        let api = ApiClient::new(server.api_url.clone(), Some("secret-token".to_string()));

        let list = execute(
            &api,
            Commands::Space {
                command: SpaceCommands::List,
            },
        )
        .await
        .unwrap();
        let create = execute(
            &api,
            Commands::Space {
                command: SpaceCommands::Create {
                    key: "SDLC KB".to_string(),
                    name: "SDLC Knowledge Base".to_string(),
                    description: Some("Internal docs".to_string()),
                },
            },
        )
        .await
        .unwrap();
        let get = execute(
            &api,
            Commands::Space {
                command: SpaceCommands::Get {
                    key: "SDLC KB".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let tree = execute(
            &api,
            Commands::Space {
                command: SpaceCommands::Tree {
                    key: "SDLC KB".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let members = execute(
            &api,
            Commands::Space {
                command: SpaceCommands::Members {
                    key: "SDLC KB".to_string(),
                },
            },
        )
        .await
        .unwrap();

        for value in [list, create, get, tree, members] {
            assert_eq!(value["status"], "ok");
        }

        let requests = server.requests();
        assert_eq!(requests.len(), 5);
        assert_eq!(requests[0].method, Method::GET);
        assert_eq!(requests[0].path, "/api/v1/spaces");
        assert!(requests[0].idempotency_key.is_none());

        let create = &requests[1];
        assert_eq!(create.method, Method::POST);
        assert_eq!(create.path, "/api/v1/spaces");
        assert_write_idempotency_key(create);
        let body: Value = serde_json::from_slice(&create.body).unwrap();
        assert_eq!(body["key"], "SDLC KB");
        assert_eq!(body["name"], "SDLC Knowledge Base");
        assert_eq!(body["description"], "Internal docs");

        assert_eq!(requests[2].method, Method::GET);
        assert_eq!(requests[2].path, "/api/v1/spaces/SDLC%20KB");
        assert!(requests[2].idempotency_key.is_none());
        assert_eq!(requests[3].method, Method::GET);
        assert_eq!(requests[3].path, "/api/v1/spaces/SDLC%20KB/tree");
        assert!(requests[3].idempotency_key.is_none());
        assert_eq!(requests[4].method, Method::GET);
        assert_eq!(requests[4].path, "/api/v1/spaces/SDLC%20KB/members");
        assert!(requests[4].idempotency_key.is_none());
    }

    #[tokio::test]
    async fn doc_create_sends_json_body_and_idempotency_key() {
        let server = spawn_mock_server().await;
        let api = ApiClient::new(server.api_url.clone(), Some("secret-token".to_string()));
        let path = temp_file("wiki-cli-doc", "# Requirements\n\nCLI body");

        let value = execute(
            &api,
            Commands::Doc {
                command: DocCommands::Create(DocCreateArgs {
                    space: "SDLC".to_string(),
                    title: "CLI Requirements".to_string(),
                    document_type: "requirements".to_string(),
                    parent_id: None,
                    slug: Some("cli-requirements".to_string()),
                    task: Some("SDLC-42".to_string()),
                    phase: Some("implementation".to_string()),
                    from_file: path.clone(),
                }),
            },
        )
        .await
        .unwrap();
        let _ = std::fs::remove_file(path);

        assert_eq!(value["status"], "ok");
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
                .idempotency_key
                .as_deref()
                .is_some_and(|value| value.starts_with("wiki-cli-write-"))
        );
        assert!(
            request
                .content_type
                .as_deref()
                .is_some_and(|value| value.starts_with("application/json"))
        );
        let body: Value = serde_json::from_slice(&request.body).unwrap();
        assert_eq!(body["title"], "CLI Requirements");
        assert_eq!(body["document_type"], "requirements");
        assert_eq!(body["slug"], "cli-requirements");
        assert_eq!(body["task_key"], "SDLC-42");
        assert_eq!(body["phase_key"], "implementation");
        assert_eq!(body["content_markdown"], "# Requirements\n\nCLI body");
    }

    #[tokio::test]
    async fn doc_lifecycle_commands_use_public_api_and_idempotency_keys() {
        let server = spawn_mock_server().await;
        let api = ApiClient::new(server.api_url.clone(), Some("secret-token".to_string()));
        let path = temp_file("wiki-cli-draft", "# Updated\n\nDraft body");

        let draft = execute(
            &api,
            Commands::Doc {
                command: DocCommands::Draft(DocContentArgs {
                    document_id: "product requirements".to_string(),
                    from_file: path.clone(),
                }),
            },
        )
        .await
        .unwrap();
        let publish = execute(
            &api,
            Commands::Doc {
                command: DocCommands::Publish {
                    document_id: "product requirements".to_string(),
                    summary: Some("Clarified scope".to_string()),
                },
            },
        )
        .await
        .unwrap();
        let archive = execute(
            &api,
            Commands::Doc {
                command: DocCommands::Archive {
                    document_id: "product requirements".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let move_document = execute(
            &api,
            Commands::Doc {
                command: DocCommands::Move {
                    document_id: "product requirements".to_string(),
                    parent: Some("parent document".to_string()),
                },
            },
        )
        .await
        .unwrap();
        let history = execute(
            &api,
            Commands::Doc {
                command: DocCommands::History {
                    document_id: "product requirements".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let _ = std::fs::remove_file(path);

        for value in [draft, publish, archive, move_document, history] {
            assert_eq!(value["status"], "ok");
        }

        let requests = server.requests();
        assert_eq!(requests.len(), 5);

        let draft = &requests[0];
        assert_eq!(draft.method, Method::PUT);
        assert_eq!(draft.path, "/api/v1/documents/product%20requirements/draft");
        assert_eq!(draft.authorization.as_deref(), Some("Bearer secret-token"));
        assert!(
            draft
                .idempotency_key
                .as_deref()
                .is_some_and(|value| value.starts_with("wiki-cli-write-"))
        );
        let body: Value = serde_json::from_slice(&draft.body).unwrap();
        assert_eq!(body["content_markdown"], "# Updated\n\nDraft body");

        let publish = &requests[1];
        assert_eq!(publish.method, Method::POST);
        assert_eq!(
            publish.path,
            "/api/v1/documents/product%20requirements/publish"
        );
        assert!(
            publish
                .idempotency_key
                .as_deref()
                .is_some_and(|value| value.starts_with("wiki-cli-write-"))
        );
        let body: Value = serde_json::from_slice(&publish.body).unwrap();
        assert_eq!(body["summary"], "Clarified scope");

        let archive = &requests[2];
        assert_eq!(archive.method, Method::POST);
        assert_eq!(
            archive.path,
            "/api/v1/documents/product%20requirements/archive"
        );
        assert!(
            archive
                .idempotency_key
                .as_deref()
                .is_some_and(|value| value.starts_with("wiki-cli-write-"))
        );
        let body: Value = serde_json::from_slice(&archive.body).unwrap();
        assert_eq!(body, json!({}));

        let move_document = &requests[3];
        assert_eq!(move_document.method, Method::POST);
        assert_eq!(
            move_document.path,
            "/api/v1/documents/product%20requirements/move"
        );
        assert!(
            move_document
                .idempotency_key
                .as_deref()
                .is_some_and(|value| value.starts_with("wiki-cli-write-"))
        );
        let body: Value = serde_json::from_slice(&move_document.body).unwrap();
        assert_eq!(body["parent_id"], "parent document");

        let history = &requests[4];
        assert_eq!(history.method, Method::GET);
        assert_eq!(
            history.path,
            "/api/v1/documents/product%20requirements/revisions"
        );
        assert!(history.idempotency_key.is_none());
    }

    #[tokio::test]
    async fn read_commands_use_public_api_paths_without_idempotency_keys() {
        let server = spawn_mock_server().await;
        let api = ApiClient::new(server.api_url.clone(), Some("secret-token".to_string()));

        let document = execute(
            &api,
            Commands::Doc {
                command: DocCommands::Get {
                    document_id: "product requirements".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let evidence = execute(
            &api,
            Commands::Evidence {
                command: EvidenceCommands::Get {
                    evidence_id: "smoke evidence".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let templates = execute(
            &api,
            Commands::Template {
                command: TemplateCommands::List,
            },
        )
        .await
        .unwrap();
        let settings = execute(
            &api,
            Commands::Settings {
                command: SettingsCommands::Get,
            },
        )
        .await
        .unwrap();

        assert_eq!(document["status"], "ok");
        assert_eq!(evidence["status"], "ok");
        assert!(templates["templates"].is_array());
        assert_eq!(settings["instance_name"], "Wiki");

        let requests = server.requests();
        assert_eq!(requests.len(), 4);
        assert_eq!(requests[0].method, Method::GET);
        assert_eq!(requests[0].path, "/api/v1/documents/product%20requirements");
        assert!(requests[0].idempotency_key.is_none());
        assert_eq!(requests[1].method, Method::GET);
        assert_eq!(requests[1].path, "/api/v1/evidence/smoke%20evidence");
        assert!(requests[1].idempotency_key.is_none());
        assert_eq!(requests[2].method, Method::GET);
        assert_eq!(requests[2].path, "/api/v1/templates");
        assert!(requests[2].idempotency_key.is_none());
        assert_eq!(requests[3].method, Method::GET);
        assert_eq!(requests[3].path, "/api/v1/settings");
        assert_eq!(
            requests[3].authorization.as_deref(),
            Some("Bearer secret-token")
        );
        assert!(requests[3].idempotency_key.is_none());
    }

    #[tokio::test]
    async fn task_and_phase_commands_use_public_api_paths() {
        let server = spawn_mock_server().await;
        let api = ApiClient::new(server.api_url.clone(), Some("secret-token".to_string()));

        let task_get = execute(
            &api,
            Commands::Task {
                command: TaskCommands::Get(LinkTargetArgs {
                    space: "SDLC KB".to_string(),
                    key: "SDLC-42".to_string(),
                }),
            },
        )
        .await
        .unwrap();
        let task_docs = execute(
            &api,
            Commands::Task {
                command: TaskCommands::Docs(LinkTargetArgs {
                    space: "SDLC KB".to_string(),
                    key: "SDLC-42".to_string(),
                }),
            },
        )
        .await
        .unwrap();
        let task_evidence = execute(
            &api,
            Commands::Task {
                command: TaskCommands::Evidence(LinkTargetArgs {
                    space: "SDLC KB".to_string(),
                    key: "SDLC-42".to_string(),
                }),
            },
        )
        .await
        .unwrap();
        let task_link = execute(
            &api,
            Commands::Task {
                command: TaskCommands::LinkDoc(LinkDocumentArgs {
                    space: "SDLC KB".to_string(),
                    key: "SDLC-42".to_string(),
                    document: "product requirements".to_string(),
                }),
            },
        )
        .await
        .unwrap();
        let phase_get = execute(
            &api,
            Commands::Phase {
                command: PhaseCommands::Get(LinkTargetArgs {
                    space: "SDLC KB".to_string(),
                    key: "implementation".to_string(),
                }),
            },
        )
        .await
        .unwrap();
        let phase_docs = execute(
            &api,
            Commands::Phase {
                command: PhaseCommands::Docs(LinkTargetArgs {
                    space: "SDLC KB".to_string(),
                    key: "implementation".to_string(),
                }),
            },
        )
        .await
        .unwrap();
        let phase_evidence = execute(
            &api,
            Commands::Phase {
                command: PhaseCommands::Evidence(LinkTargetArgs {
                    space: "SDLC KB".to_string(),
                    key: "implementation".to_string(),
                }),
            },
        )
        .await
        .unwrap();
        let phase_link = execute(
            &api,
            Commands::Phase {
                command: PhaseCommands::LinkDoc(LinkDocumentArgs {
                    space: "SDLC KB".to_string(),
                    key: "implementation".to_string(),
                    document: "product requirements".to_string(),
                }),
            },
        )
        .await
        .unwrap();

        for value in [
            task_get,
            task_docs,
            task_evidence,
            task_link,
            phase_get,
            phase_docs,
            phase_evidence,
            phase_link,
        ] {
            assert_eq!(value["status"], "ok");
        }

        let requests = server.requests();
        assert_eq!(requests.len(), 8);
        assert_eq!(requests[0].method, Method::GET);
        assert_eq!(requests[0].path, "/api/v1/spaces/SDLC%20KB/tasks/SDLC-42");
        assert!(requests[0].idempotency_key.is_none());
        assert_eq!(requests[1].method, Method::GET);
        assert_eq!(
            requests[1].path,
            "/api/v1/spaces/SDLC%20KB/tasks/SDLC-42/documents"
        );
        assert_eq!(requests[2].method, Method::GET);
        assert_eq!(
            requests[2].path,
            "/api/v1/spaces/SDLC%20KB/tasks/SDLC-42/evidence"
        );

        let task_link = &requests[3];
        assert_eq!(task_link.method, Method::POST);
        assert_eq!(
            task_link.path,
            "/api/v1/spaces/SDLC%20KB/tasks/SDLC-42/links/documents"
        );
        assert_write_idempotency_key(task_link);
        let body: Value = serde_json::from_slice(&task_link.body).unwrap();
        assert_eq!(body["document_id"], "product requirements");

        assert_eq!(requests[4].method, Method::GET);
        assert_eq!(
            requests[4].path,
            "/api/v1/spaces/SDLC%20KB/phases/implementation"
        );
        assert!(requests[4].idempotency_key.is_none());
        assert_eq!(requests[5].method, Method::GET);
        assert_eq!(
            requests[5].path,
            "/api/v1/spaces/SDLC%20KB/phases/implementation/documents"
        );
        assert_eq!(requests[6].method, Method::GET);
        assert_eq!(
            requests[6].path,
            "/api/v1/spaces/SDLC%20KB/phases/implementation/evidence"
        );

        let phase_link = &requests[7];
        assert_eq!(phase_link.method, Method::POST);
        assert_eq!(
            phase_link.path,
            "/api/v1/spaces/SDLC%20KB/phases/implementation/links/documents"
        );
        assert_write_idempotency_key(phase_link);
        let body: Value = serde_json::from_slice(&phase_link.body).unwrap();
        assert_eq!(body["document_id"], "product requirements");
    }

    #[tokio::test]
    async fn add_link_sends_document_task_phase_json() {
        let server = spawn_mock_server().await;
        let api = ApiClient::new(server.api_url.clone(), Some("secret-token".to_string()));

        let value = execute(
            &api,
            Commands::Evidence {
                command: EvidenceCommands::AddLink(EvidenceLinkArgs {
                    space: Some("SDLC".to_string()),
                    document: Some("product-requirements".to_string()),
                    task: Some("SDLC-42".to_string()),
                    phase: Some("testing".to_string()),
                    evidence_type: "external_url".to_string(),
                    title: "CLI link evidence".to_string(),
                    url: "https://ci.local/jobs/42".to_string(),
                }),
            },
        )
        .await
        .unwrap();

        assert_eq!(value["status"], "ok");
        let requests = server.requests();
        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.path, "/api/v1/evidence");
        assert!(
            request
                .idempotency_key
                .as_deref()
                .is_some_and(|value| value.starts_with("wiki-cli-write-"))
        );
        let body: Value = serde_json::from_slice(&request.body).unwrap();
        assert_eq!(body["space"], "SDLC");
        assert_eq!(body["document_id"], "product-requirements");
        assert_eq!(body["task_key"], "SDLC-42");
        assert_eq!(body["phase_key"], "testing");
        assert_eq!(body["evidence_type"], "external_url");
        assert_eq!(body["title"], "CLI link evidence");
        assert_eq!(body["url"], "https://ci.local/jobs/42");
    }

    #[tokio::test]
    async fn add_file_uploads_attachment_then_creates_file_evidence() {
        let server = spawn_mock_server().await;
        let api = ApiClient::new(server.api_url.clone(), Some("secret-token".to_string()));
        let path = temp_file("wiki-cli-evidence", "file evidence bytes");

        let value = execute(
            &api,
            Commands::Evidence {
                command: EvidenceCommands::AddFile(EvidenceFileArgs {
                    space: Some("SDLC".to_string()),
                    document: Some("product-requirements".to_string()),
                    task: Some("SDLC-42".to_string()),
                    phase: Some("testing".to_string()),
                    evidence_type: "uploaded_file".to_string(),
                    title: "CLI file evidence".to_string(),
                    file: path.clone(),
                }),
            },
        )
        .await
        .unwrap();
        let _ = std::fs::remove_file(path);

        assert_eq!(value["status"], "ok");
        let requests = server.requests();
        assert_eq!(requests.len(), 2);

        let upload = &requests[0];
        assert_eq!(upload.method, Method::POST);
        assert_eq!(upload.path, "/api/v1/attachments");
        assert!(
            upload
                .idempotency_key
                .as_deref()
                .is_some_and(|value| value.starts_with("wiki-cli-upload-"))
        );
        assert!(
            upload
                .content_type
                .as_deref()
                .is_some_and(|value| value.starts_with("multipart/form-data"))
        );

        let evidence = &requests[1];
        assert_eq!(evidence.method, Method::POST);
        assert_eq!(evidence.path, "/api/v1/evidence");
        assert!(
            evidence
                .idempotency_key
                .as_deref()
                .is_some_and(|value| value.starts_with("wiki-cli-write-"))
        );
        let body: Value = serde_json::from_slice(&evidence.body).unwrap();
        assert_eq!(body["space"], "SDLC");
        assert_eq!(body["document_id"], "product-requirements");
        assert_eq!(body["task_key"], "SDLC-42");
        assert_eq!(body["phase_key"], "testing");
        assert_eq!(body["evidence_type"], "uploaded_file");
        assert_eq!(body["title"], "CLI file evidence");
        assert_eq!(body["attachment_id"], "attachment-1");
        assert_eq!(body["checksum"], "sha256-test");
    }

    #[tokio::test]
    async fn evidence_list_builds_owner_filter_query() {
        let server = spawn_mock_server().await;
        let api = ApiClient::new(server.api_url.clone(), Some("secret-token".to_string()));

        let value = execute(
            &api,
            Commands::Evidence {
                command: EvidenceCommands::List {
                    space: Some("SDLC KB".to_string()),
                    document: Some("product-requirements".to_string()),
                    task: Some("SDLC-42".to_string()),
                    phase: Some("testing".to_string()),
                },
            },
        )
        .await
        .unwrap();

        assert_eq!(value["status"], "ok");
        let requests = server.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, Method::GET);
        assert_eq!(
            requests[0].path,
            "/api/v1/evidence?space=SDLC%20KB&document_id=product-requirements&task_key=SDLC-42&phase_key=testing"
        );
        assert_eq!(
            requests[0].authorization.as_deref(),
            Some("Bearer secret-token")
        );
        assert!(requests[0].idempotency_key.is_none());
    }

    #[tokio::test]
    async fn template_apply_matches_document_type_and_creates_document() {
        let server = spawn_mock_server().await;
        let api = ApiClient::new(server.api_url.clone(), Some("secret-token".to_string()));

        let value = execute(
            &api,
            Commands::Template {
                command: TemplateCommands::Apply(TemplateApplyArgs {
                    template: "requirements".to_string(),
                    space: "SDLC".to_string(),
                    title: "CLI template document".to_string(),
                    parent_id: Some("parent-document".to_string()),
                    task: Some("SDLC-42".to_string()),
                    phase: Some("requirements".to_string()),
                }),
            },
        )
        .await
        .unwrap();

        assert_eq!(value["status"], "ok");
        let requests = server.requests();
        assert_eq!(requests.len(), 2);

        let list_templates = &requests[0];
        assert_eq!(list_templates.method, Method::GET);
        assert_eq!(list_templates.path, "/api/v1/templates");
        assert!(list_templates.idempotency_key.is_none());

        let create_document = &requests[1];
        assert_eq!(create_document.method, Method::POST);
        assert_eq!(create_document.path, "/api/v1/spaces/SDLC/documents");
        assert!(
            create_document
                .idempotency_key
                .as_deref()
                .is_some_and(|value| value.starts_with("wiki-cli-write-"))
        );
        let body: Value = serde_json::from_slice(&create_document.body).unwrap();
        assert_eq!(body["title"], "CLI template document");
        assert_eq!(body["document_type"], "requirements");
        assert_eq!(body["parent_id"], "parent-document");
        assert_eq!(body["task_key"], "SDLC-42");
        assert_eq!(body["phase_key"], "requirements");
        assert_eq!(body["content_markdown"], "# Requirements\n\nTemplate body");
    }

    fn temp_file(prefix: &str, content: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("{prefix}-{}.md", idempotency_key("test")));
        std::fs::write(&path, content).unwrap();
        path
    }
}
