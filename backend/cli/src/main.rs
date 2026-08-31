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
    },
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
            bail!("API returned {status}: {text}");
        }
        if text.trim().is_empty() {
            return Ok(json!({ "status": "ok" }));
        }
        serde_json::from_str(&text).context("API returned non-JSON response")
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
        EvidenceCommands::List { space, task, phase } => {
            let query = query_string([("space", space), ("task_key", task), ("phase_key", phase)]);
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
        } => {
            let query = query_string([
                ("q", Some(query)),
                ("space", space),
                ("task_key", task),
                ("phase_key", phase),
            ]);
            api.get(&format!("/search{}", query)).await
        }
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
