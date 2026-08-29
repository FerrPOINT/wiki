use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use reqwest::{Client, Method, multipart};
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "wiki")]
#[command(about = "Wiki CLI -- manage spaces, documents, task dossiers and evidence")]
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
    Whoami,
    TokenCreate {
        #[arg(long)]
        name: String,
        #[arg(long)]
        scope: Vec<String>,
    },
    TokenRevoke {
        token_id: String,
    },
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
    Approve {
        document_id: String,
    },
    Archive {
        document_id: String,
    },
    Restore {
        document_id: String,
    },
    History {
        document_id: String,
    },
    Diff {
        document_id: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
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
    Upsert {
        #[arg(long)]
        space: String,
        #[arg(long)]
        source: String,
        #[arg(long)]
        key: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },
    Get {
        #[arg(long)]
        space: String,
        #[arg(long)]
        source: String,
        #[arg(long)]
        key: String,
    },
    Docs {
        task_key: String,
    },
    Phases {
        task_key: String,
    },
}

#[derive(Subcommand)]
enum PhaseCommands {
    Upsert {
        #[arg(long)]
        task: String,
        #[arg(long)]
        workflow_run: Option<String>,
        #[arg(long)]
        phase_code: String,
        #[arg(long)]
        phase_name: String,
        #[arg(long)]
        status: Option<String>,
    },
    Complete {
        #[arg(long)]
        task: String,
        #[arg(long)]
        phase: String,
        #[arg(long)]
        verdict: Option<String>,
    },
    Docs {
        #[arg(long)]
        task: String,
        #[arg(long)]
        phase: String,
    },
    Evidence {
        #[arg(long)]
        task: String,
        #[arg(long)]
        phase: String,
    },
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
        task: Option<String>,
        #[arg(long)]
        phase: Option<String>,
    },
}

#[derive(Args)]
struct EvidenceLinkArgs {
    #[arg(long)]
    task: Option<String>,
    #[arg(long)]
    phase: Option<String>,
    #[arg(long = "type", default_value = "link")]
    evidence_type: String,
    #[arg(long)]
    title: String,
    #[arg(long)]
    url: String,
}

#[derive(Args)]
struct EvidenceFileArgs {
    #[arg(long)]
    task: Option<String>,
    #[arg(long)]
    phase: Option<String>,
    #[arg(long = "type", default_value = "file")]
    evidence_type: String,
    #[arg(long)]
    title: String,
    #[arg(long)]
    file: PathBuf,
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

    async fn delete(&self, path: &str) -> Result<Value> {
        self.request(Method::DELETE, path).send_json().await
    }

    async fn post_json(&self, path: &str, body: Value) -> Result<Value> {
        self.request(Method::POST, path)
            .json(&body)
            .send_json()
            .await
    }

    async fn put_json(&self, path: &str, body: Value) -> Result<Value> {
        self.request(Method::PUT, path)
            .json(&body)
            .send_json()
            .await
    }

    async fn post_multipart(&self, path: &str, form: multipart::Form) -> Result<Value> {
        self.request(Method::POST, path)
            .multipart(form)
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
        AuthCommands::Whoami => api.get("/users/me").await,
        AuthCommands::TokenCreate { name, scope } => {
            api.post_json("/users/me/tokens", json!({ "name": name, "scopes": scope }))
                .await
        }
        AuthCommands::TokenRevoke { token_id } => {
            api.delete(&format!("/users/me/tokens/{}", enc(&token_id)))
                .await
        }
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
    }
}

async fn execute_doc(api: &ApiClient, command: DocCommands) -> Result<Value> {
    match command {
        DocCommands::Create(args) => {
            let content = read_file(&args.from_file).await?;
            api.post_json(
                &format!("/spaces/{}/documents", enc(&args.space)),
                json!({
                    "title": args.title,
                    "document_type": args.document_type,
                    "parent_id": args.parent_id,
                    "slug": args.slug,
                    "task_key": args.task,
                    "phase_code": args.phase,
                    "content_markdown": content
                }),
            )
            .await
        }
        DocCommands::Get { document_id } => {
            api.get(&format!("/documents/{}", enc(&document_id))).await
        }
        DocCommands::Draft(args) => {
            let content = read_file(&args.from_file).await?;
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
                json!({ "change_summary": summary }),
            )
            .await
        }
        DocCommands::Approve { document_id } => {
            api.post_json(
                &format!("/documents/{}/approve", enc(&document_id)),
                json!({}),
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
        DocCommands::Restore { document_id } => {
            api.post_json(
                &format!("/documents/{}/restore", enc(&document_id)),
                json!({}),
            )
            .await
        }
        DocCommands::History { document_id } => {
            api.get(&format!("/documents/{}/revisions", enc(&document_id)))
                .await
        }
        DocCommands::Diff {
            document_id,
            from,
            to,
        } => {
            api.get(&format!(
                "/documents/{}/diff?from={}&to={}",
                enc(&document_id),
                enc(&from),
                enc(&to)
            ))
            .await
        }
    }
}

async fn execute_task(api: &ApiClient, command: TaskCommands) -> Result<Value> {
    match command {
        TaskCommands::Upsert {
            space,
            source,
            key,
            title,
            url,
            status,
        } => {
            api.post_json(
                &format!("/spaces/{}/tasks", enc(&space)),
                json!({
                    "source_system": source,
                    "external_task_key": key,
                    "title_snapshot": title,
                    "external_task_url": url,
                    "status_snapshot": status
                }),
            )
            .await
        }
        TaskCommands::Get { space, source, key } => {
            api.get(&format!(
                "/spaces/{}/tasks/{}/{}",
                enc(&space),
                enc(&source),
                enc(&key)
            ))
            .await
        }
        TaskCommands::Docs { task_key } => {
            api.get(&format!("/task-dossiers/{}/documents", enc(&task_key)))
                .await
        }
        TaskCommands::Phases { task_key } => {
            api.get(&format!("/task-dossiers/{}/phases", enc(&task_key)))
                .await
        }
    }
}

async fn execute_phase(api: &ApiClient, command: PhaseCommands) -> Result<Value> {
    match command {
        PhaseCommands::Upsert {
            task,
            workflow_run,
            phase_code,
            phase_name,
            status,
        } => {
            api.post_json(
                &format!("/task-dossiers/{}/phases", enc(&task)),
                json!({
                    "workflow_run_id": workflow_run,
                    "phase_code": phase_code,
                    "phase_name": phase_name,
                    "phase_status": status
                }),
            )
            .await
        }
        PhaseCommands::Complete {
            task,
            phase,
            verdict,
        } => {
            api.post_json(
                &format!(
                    "/task-dossiers/{}/phases/{}/complete",
                    enc(&task),
                    enc(&phase)
                ),
                json!({ "supervisor_verdict": verdict }),
            )
            .await
        }
        PhaseCommands::Docs { task, phase } => {
            api.get(&format!(
                "/task-dossiers/{}/phases/{}/documents",
                enc(&task),
                enc(&phase)
            ))
            .await
        }
        PhaseCommands::Evidence { task, phase } => {
            api.get(&format!(
                "/task-dossiers/{}/phases/{}/evidence",
                enc(&task),
                enc(&phase)
            ))
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
                    "task_key": args.task,
                    "phase_code": args.phase,
                    "evidence_type": args.evidence_type,
                    "title": args.title,
                    "url": args.url
                }),
            )
            .await
        }
        EvidenceCommands::AddFile(args) => {
            let bytes = tokio::fs::read(&args.file)
                .await
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
                    "task_key": args.task,
                    "phase_code": args.phase,
                    "evidence_type": args.evidence_type,
                    "title": args.title,
                    "filename": filename,
                    "attachment": attachment
                }),
            )
            .await
        }
        EvidenceCommands::Get { evidence_id } => {
            api.get(&format!("/evidence/{}", enc(&evidence_id))).await
        }
        EvidenceCommands::List { task, phase } => {
            let query = query_string([("task_key", task), ("phase_code", phase)]);
            api.get(&format!("/evidence{}", query)).await
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
                ("phase_code", phase),
            ]);
            api.get(&format!("/search{}", query)).await
        }
    }
}

async fn read_file(path: &PathBuf) -> Result<String> {
    tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("failed to read {}", path.display()))
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
                "title",
                "name",
                "status",
                "phase_code",
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
