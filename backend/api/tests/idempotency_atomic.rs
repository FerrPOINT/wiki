//! Manual HTTP/PostgreSQL command-outcome boundary, isolated schema only.
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tower::ServiceExt;
use uuid::Uuid;
#[path = "../../infra/tests/support/commit_loss.rs"]
mod commit_loss;
use commit_loss::FaultProxy;
async fn call(
    app: &Router,
    path: &str,
    token: Option<&str>,
    key: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut request = Request::post(path).header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    if let Some(key) = key {
        request = request.header("Idempotency-Key", key);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 8 * 1024 * 1024)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
// These manual PostgreSQL tests run with --test-threads=1. Restore the
// deployment reader environment after the owning backend has been closed.
struct TrackerFixture {
    project: shared::resource_context::ResourceContextSummary,
    task: Uuid,
    server: tokio::task::JoinHandle<()>,
    token_path: std::path::PathBuf,
    previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
}
impl TrackerFixture {
    async fn new() -> Self {
        use shared::resource_context::*;
        let tracker = Uuid::new_v4();
        let command = OwnerCommand {
            schema_version: 1,
            namespace: NamespaceRef {
                registry_instance_id: Uuid::new_v4(),
                namespace_id: Uuid::new_v4(),
            },
            resource: ResourceRef {
                kind: ResourceKind::TrackerProject,
                instance_id: tracker,
                resource_id: Uuid::new_v4(),
            },
            operation_id: Uuid::new_v4(),
            generation: 1,
            state: "active".into(),
            create_spec: None,
        };
        let project = ResourceContextSummary {
            binding: command.readback(true),
            label: "Matrix project".into(),
            resource_key: "MATRIX".into(),
        };
        let task = Uuid::new_v4();
        let readback = json!({"schema_version":1,"task_id":task,"tracker_instance_id":tracker,
            "state":"active","namespace":command.namespace,"task_key":"MATRIX-1"});
        let api = Router::new()
            .route(
                "/api/v1/namespace-projects",
                axum::routing::get({
                    let project = project.clone();
                    move || async move { axum::Json(vec![project]) }
                }),
            )
            .route(
                &format!("/api/v1/namespace-tasks/{task}"),
                axum::routing::get(move |headers: axum::http::HeaderMap| async move {
                    assert_eq!(
                        headers.get("authorization").unwrap(),
                        "Bearer synthetic-reader"
                    );
                    axum::Json(readback)
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}/", listener.local_addr().unwrap());
        let server = tokio::spawn(async move { axum::serve(listener, api).await.unwrap() });
        let token_path = std::env::temp_dir().join(format!("wiki-reader-{}", Uuid::new_v4()));
        std::fs::write(&token_path, "synthetic-reader").unwrap();
        let mut previous = Vec::new();
        for (key, value) in [
            ("WIKI_NAMESPACE__TRACKER_URL", origin),
            ("WIKI_NAMESPACE__TRACKER_INSTANCE_ID", tracker.to_string()),
            (
                "WIKI_NAMESPACE__REGISTRY_INSTANCE_ID",
                command.namespace.registry_instance_id.to_string(),
            ),
            (
                "WIKI_NAMESPACE__TRACKER_TOKEN_FILE",
                token_path.to_string_lossy().into_owned(),
            ),
        ] {
            previous.push((key, std::env::var_os(key)));
            // No parallel test reads this environment in the manual command.
            unsafe { std::env::set_var(key, value) };
        }
        Self {
            project,
            task,
            server,
            token_path,
            previous,
        }
    }
}
impl Drop for TrackerFixture {
    fn drop(&mut self) {
        self.server.abort();
        for (key, value) in self.previous.drain(..) {
            unsafe {
                if let Some(value) = value {
                    std::env::set_var(key, value)
                } else {
                    std::env::remove_var(key)
                }
            }
        }
        let _ = std::fs::remove_file(&self.token_path);
    }
}
struct Fixture {
    root: PgPool,
    control: PgPool,
    router: Router,
    schema: String,
    path: std::path::PathBuf,
    proxy: Option<FaultProxy>,
}
impl Fixture {
    async fn new() -> Self {
        Self::with_fault(None, false).await
    }
    async fn with_fault(refuse_readback: Option<bool>, drop_failure: bool) -> Self {
        let raw = std::env::var("WIKI_TEST_DATABASE_URL").unwrap();
        let root = PgPool::connect(&raw).await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, String>("SELECT current_database()")
                .fetch_one(&root)
                .await
                .unwrap(),
            "wiki_space_test"
        );
        let schema = format!("wiki_result_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&root)
            .await
            .unwrap();
        let mut scoped = reqwest::Url::parse(&raw).unwrap();
        scoped
            .query_pairs_mut()
            .append_pair("options", &format!("-csearch_path={schema}"));
        let path = std::env::temp_dir().join(format!("wiki-result-{}", Uuid::new_v4().simple()));
        let proxy = if let Some(refuse) = refuse_readback {
            let socket = std::env::var("WIKI_TEST_POSTGRES_SOCKET").unwrap() + "/.s.PGSQL.5432";
            Some(FaultProxy::start(socket, refuse, drop_failure).await)
        } else {
            None
        };
        let mut config = shared::AppConfig::default();
        // The writer matrix deliberately exceeds the ordinary 60-request burst;
        // rate-limit behavior is covered separately, not by waiting a minute here.
        config.server.general_rate_burst = 256;
        config.database.url = if let Some(proxy) = proxy.as_ref() {
            let mut url = reqwest::Url::parse(&format!(
                "postgres://qa@127.0.0.1:{}/wiki_space_test?sslmode=disable",
                proxy.port
            ))
            .unwrap();
            url.query_pairs_mut()
                .append_pair("options", &format!("-csearch_path={schema}"));
            url.to_string()
        } else {
            scoped.to_string()
        };
        config.database.connect_timeout_seconds = 1;
        config.database.max_connections = 4;
        config.database.min_connections = 0;
        config.auth.jwt_secret = "synthetic-transaction-result-test-only".into();
        config.bootstrap.admin_email = Some("result@example.test".into());
        config.bootstrap.admin_username = Some("result".into());
        config.bootstrap.admin_password = Some("Synthetic result fixture 42!".into());
        config.storage.dir = path.to_string_lossy().into_owned();
        let config = Arc::new(config);
        let ctx = Arc::new(app::WikiAppContext::new(config.clone()));
        let (backend, settings) = infra::connect_postgres_wiki_backend(
            &config,
            Arc::new(infra::LocalWikiAttachmentStorage::new(&path)),
        )
        .await
        .unwrap();
        let router = api::router_with_wiki(
            ctx.clone(),
            api::routes::wiki::WikiBackend::persistent(backend, settings),
        )
        .with_state(ctx);
        let control = PgPoolOptions::new()
            .max_connections(3)
            .connect(scoped.as_str())
            .await
            .unwrap();
        Self {
            root,
            control,
            router,
            schema,
            path,
            proxy,
        }
    }
    async fn close(self) {
        let Self {
            root,
            control,
            router,
            schema,
            path,
            proxy,
        } = self;
        drop(router);
        drop(proxy);
        control.close().await;
        if path.exists() {
            std::fs::remove_dir_all(path).unwrap();
        }
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&root)
            .await
            .unwrap();
        root.close().await;
    }
}
async fn send(
    app: &Router,
    method: &str,
    path: &str,
    token: &str,
    key: Option<&str>,
    content_type: &str,
    bytes: Vec<u8>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", content_type)
        .header("authorization", format!("Bearer {token}"));
    if let Some(key) = key {
        request = request.header("Idempotency-Key", key);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(bytes)).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 8 * 1024 * 1024)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
async fn snapshot(pool: &PgPool) -> Value {
    let mut result = serde_json::Map::new();
    for table in [
        "users",
        "wiki_confirmed_global_roles",
        "spaces",
        "space_members",
        "documents",
        "document_drafts",
        "document_revisions",
        "document_task_links",
        "document_phase_links",
        "task_dossiers",
        "phase_dossiers",
        "evidence_items",
        "attachments",
        "document_templates",
        "audit_log",
        "wiki_space_projects",
        "managed_task_document_links",
    ] {
        let rows:Value=sqlx::query_scalar(&format!("SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text),'[]'::jsonb) FROM {table} t"))
            .fetch_one(pool).await.unwrap();
        result.insert(table.into(), rows);
    }
    Value::Object(result)
}
struct Commands<'a> {
    app: &'a Router,
    pool: &'a PgPool,
    token: &'a str,
}
impl Commands<'_> {
    async fn json(
        &self,
        label: &str,
        method: &str,
        path: &str,
        payload: Value,
        status: StatusCode,
    ) -> Value {
        self.bytes(
            label,
            method,
            path,
            "application/json",
            payload.to_string().into_bytes(),
            status,
        )
        .await
    }
    async fn bytes(
        &self,
        label: &str,
        method: &str,
        path: &str,
        content_type: &str,
        bytes: Vec<u8>,
        status: StatusCode,
    ) -> Value {
        let Self { app, pool, token } = *self;
        let before = snapshot(pool).await;
        sqlx::raw_sql("CREATE OR REPLACE FUNCTION refuse_command_result() RETURNS trigger LANGUAGE plpgsql AS $$BEGIN RAISE EXCEPTION 'fixture result persistence refusal'; END$$; CREATE TRIGGER refuse_command_result BEFORE UPDATE ON idempotency_records FOR EACH ROW WHEN(NEW.state='completed') EXECUTE FUNCTION refuse_command_result()")
        .execute(pool).await.unwrap();
        let (failed, error) = send(
            app,
            method,
            path,
            token,
            Some(label),
            content_type,
            bytes.clone(),
        )
        .await;
        assert!(
            failed.is_server_error(),
            "{label}: expected result-persistence failure, got {failed}: {error}"
        );
        assert_eq!(
            snapshot(pool).await,
            before,
            "{label}: failed result persistence must roll back all subject and audit writes"
        );
        sqlx::query("DROP TRIGGER refuse_command_result ON idempotency_records")
            .execute(pool)
            .await
            .unwrap();
        let (actual, first) = send(
            app,
            method,
            path,
            token,
            Some(label),
            content_type,
            bytes.clone(),
        )
        .await;
        assert_eq!(actual, status, "{label}: {first}");
        let committed = snapshot(pool).await;
        let (actual, replayed) =
            send(app, method, path, token, Some(label), content_type, bytes).await;
        assert_eq!(actual, status, "{label}: {replayed}");
        assert_eq!(replayed, first, "{label}: exact saved response");
        assert_eq!(
            snapshot(pool).await,
            committed,
            "{label}: retry must not repeat mutations or audit"
        );
        println!("owner result transaction: {label} PASS");
        first
    }
}

#[tokio::test]
#[ignore = "manual: actual owned PostgreSQL and HTTP"]
async fn result_persistence_failure_rolls_back_the_domain_mutation_and_audit() {
    let fixture = Fixture::new().await;
    let router = fixture.router.clone();
    let control = fixture.control.clone();
    let result=tokio::spawn(async move{
        let (status,login)=call(&router,"/api/v1/auth/login",None,None,json!({"email":"result@example.test","password":"Synthetic result fixture 42!"})).await;
        assert_eq!(status,StatusCode::OK,"{login}");
        let token=login["access_token"].as_str().unwrap();
        let (status,space)=call(&router,"/api/v1/spaces",Some(token),None,json!({"key":"R13","name":"Result transaction"})).await;
        assert_eq!(status,StatusCode::CREATED,"{space}");
        let before:i64=sqlx::query_scalar("SELECT count(*) FROM audit_log").fetch_one(&control).await.unwrap();
        sqlx::raw_sql("CREATE FUNCTION refuse_command_result() RETURNS trigger LANGUAGE plpgsql AS $$BEGIN RAISE EXCEPTION 'fixture result persistence refusal'; END$$; CREATE TRIGGER refuse_command_result BEFORE UPDATE ON idempotency_records FOR EACH ROW WHEN(NEW.state='completed') EXECUTE FUNCTION refuse_command_result()")
            .execute(&control).await.unwrap();
        let payload=json!({"title":"Atomic result","content_markdown":"Preserve transactional outcome"});
        let (status,_)=call(&router,"/api/v1/spaces/R13/documents",Some(token),Some("result-key"),payload.clone()).await;
        assert!(status.is_server_error());
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM documents WHERE title='Atomic result'").fetch_one(&control).await.unwrap(),0,
            "result persistence failure must roll back the document, not leave a committed command without its outcome");
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM audit_log").fetch_one(&control).await.unwrap(),before);
        sqlx::query("DROP TRIGGER refuse_command_result ON idempotency_records").execute(&control).await.unwrap();
        let (status,first)=call(&router,"/api/v1/spaces/R13/documents",Some(token),Some("result-key"),payload.clone()).await;
        assert_eq!(status,StatusCode::CREATED,"{first}");
        let (status,replayed)=call(&router,"/api/v1/spaces/R13/documents",Some(token),Some("result-key"),payload).await;
        assert_eq!(status,StatusCode::CREATED,"{replayed}");assert_eq!(first,replayed);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM documents WHERE title='Atomic result'").fetch_one(&control).await.unwrap(),1);
        let (status,_)=call(&router,"/api/v1/spaces/R13/documents",Some(token),Some("result-key"),json!({"title":"Different","content_markdown":""})).await;
        assert_eq!(status,StatusCode::CONFLICT);
        control.close().await;
    }).await;
    fixture.close().await;
    result.unwrap();
}

#[tokio::test]
#[ignore = "manual: complete protected content writer matrix on actual PostgreSQL"]
async fn all_content_writers_bind_results_to_their_domain_transaction() {
    let reader = TrackerFixture::new().await;
    let project = reader.project.clone();
    let task = reader.task;
    let fixture = Fixture::new().await;
    let app = fixture.router.clone();
    let pool = fixture.control.clone();
    let result=tokio::spawn(async move{
        let (status,login)=call(&app,"/api/v1/auth/login",None,None,json!({"email":"result@example.test","password":"Synthetic result fixture 42!"})).await;
        assert_eq!(status,StatusCode::OK);let token=login["access_token"].as_str().unwrap();
        let commands=Commands{app:&app,pool:&pool,token};
        let user=commands.json("user-create","POST","/api/v1/users",
            json!({"email":"member@example.test","username":"member","password":"Synthetic member fixture 42!","display_name":"Member","role":"user"}),StatusCode::CREATED).await;
        let user=user["id"].as_str().unwrap();
        commands.json("user-update","PUT",&format!("/api/v1/users/{user}"),json!({"display_name":"Renamed"}),StatusCode::OK).await;
        commands.json("space-create","POST","/api/v1/spaces",json!({"key":"R13","name":"Writer matrix"}),StatusCode::CREATED).await;
        commands.json("space-update","PUT","/api/v1/spaces/R13",json!({"name":"Updated"}),StatusCode::OK).await;
        commands.json("member-upsert","PUT",&format!("/api/v1/spaces/R13/members/{user}"),json!({"role":"editor"}),StatusCode::OK).await;
        commands.json("member-delete","DELETE",&format!("/api/v1/spaces/R13/members/{user}"),Value::Null,StatusCode::NO_CONTENT).await;
        let document=commands.json("document-create","POST","/api/v1/spaces/R13/documents",json!({"title":"Matrix document","content_markdown":"Initial"}),StatusCode::CREATED).await;
        let document=document["id"].as_str().unwrap();
        commands.json("draft-update","PUT",&format!("/api/v1/documents/{document}/draft"),json!({"content_markdown":"Saved draft","title":"Saved"}),StatusCode::OK).await;
        commands.json("document-publish","POST",&format!("/api/v1/documents/{document}/publish"),json!({"base_revision_id":null,"summary":"Published"}),StatusCode::OK).await;
        let (status,parent)=call(&app,"/api/v1/spaces/R13/documents",Some(token),None,json!({"title":"Parent"})).await;
        assert_eq!(status,StatusCode::CREATED);
        commands.json("document-move","POST",&format!("/api/v1/documents/{document}/move"),json!({"parent_id":parent["id"]}),StatusCode::OK).await;
        commands.json("task-link","POST","/api/v1/spaces/R13/tasks/TASK-1/links/documents",json!({"document_id":document}),StatusCode::OK).await;
        commands.json("phase-link","POST","/api/v1/spaces/R13/phases/phase-1/links/documents",json!({"document_id":document}),StatusCode::OK).await;
        commands.json("evidence-create","POST","/api/v1/evidence",json!({"space":"R13","document_id":document,"title":"Evidence","evidence_type":"external_url","url":"https://fixture.example.test/result"}),StatusCode::CREATED).await;
        commands.json("template-create","POST","/api/v1/templates",json!({"name":"Fixture template","document_type":"requirements","body_markdown":"Template"}),StatusCode::CREATED).await;
        let bytes=b"--fixture-boundary\r\nContent-Disposition: form-data; name=\"file\"; filename=\"fixture.txt\"\r\nContent-Type: text/plain\r\n\r\nfixture bytes\r\n--fixture-boundary--\r\n".to_vec();
        commands.bytes("attachment-upload","POST","/api/v1/attachments","multipart/form-data; boundary=fixture-boundary",bytes,StatusCode::CREATED).await;
        let registry=project.binding.namespace.registry_instance_id;
        let namespace=project.binding.namespace.namespace_id;
        let tracker=project.binding.resource.instance_id;
        sqlx::query("INSERT INTO wiki_workspace_projects(registry_instance_id,namespace_id,tracker_instance_id,tracker_project_id,generation,presentation,name,project_key) VALUES($1,$2,$3,$4,1,$5,'Matrix project','MATRIX') ON CONFLICT(registry_instance_id,namespace_id) DO UPDATE SET observed_at=now(),present=true")
            .bind(registry).bind(namespace).bind(tracker).bind(project.binding.resource.resource_id).bind(json!(project)).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO wiki_workspace_catalog_sync(id,available) VALUES(true,true) ON CONFLICT(id) DO UPDATE SET available=true").execute(&pool).await.unwrap();
        let path=format!("/api/v1/spaces/R13/projects/{registry}/{namespace}");
        commands.json("legacy-link","PUT",&path,Value::Null,StatusCode::NO_CONTENT).await;
        commands.json("legacy-link-noop","PUT",&path,Value::Null,StatusCode::NO_CONTENT).await;
        let revision:Uuid=sqlx::query_scalar("SELECT id FROM document_revisions WHERE document_id=$1 ORDER BY version DESC LIMIT 1")
            .bind(Uuid::parse_str(document).unwrap()).fetch_one(&pool).await.unwrap();
        let link=json!({"namespace":project.binding.namespace,"task":{"tracker_instance_id":tracker,"task_id":task},
            "document_id":document,"revision_id":revision});
        commands.json("managed-link","POST","/api/v1/spaces/R13/managed-task-links",link.clone(),StatusCode::OK).await;
        commands.json("managed-link-existing","POST","/api/v1/spaces/R13/managed-task-links",link,StatusCode::OK).await;
        commands.json("legacy-unlink","DELETE",&path,Value::Null,StatusCode::NO_CONTENT).await;
        commands.json("document-archive","POST",&format!("/api/v1/documents/{document}/archive"),Value::Null,StatusCode::OK).await;
        commands.json("space-archive","POST","/api/v1/spaces/R13/archive",Value::Null,StatusCode::OK).await;
        pool.close().await;
    }).await;
    fixture.close().await;
    result.unwrap();
}

#[tokio::test]
#[ignore = "manual: actual COMMIT transport loss and owner-row reconciliation"]
async fn lost_commit_replies_never_repeat_a_command_or_claim_unknown_success() {
    for (refuse, rollback) in [(false, false), (true, false), (false, true), (true, true)] {
        let fixture = Fixture::with_fault(Some(refuse), rollback).await;
        let router = fixture.router.clone();
        let pool = fixture.control.clone();
        let proxy = fixture.proxy.as_ref().unwrap();
        let (armed, lost, refusal) = (
            proxy.armed.clone(),
            proxy.lost.clone(),
            proxy.refuse_readback.clone(),
        );
        let result=tokio::spawn(async move{
            let (status,login)=call(&router,"/api/v1/auth/login",None,None,json!({"email":"result@example.test","password":"Synthetic result fixture 42!"})).await;
            assert_eq!(status,StatusCode::OK);let token=login["access_token"].as_str().unwrap();
            let (status,space)=call(&router,"/api/v1/spaces",Some(token),None,json!({"key":"WIRE","name":"Wire result"})).await;
            assert_eq!(status,StatusCode::CREATED,"{space}");
            if rollback {
                sqlx::raw_sql("CREATE SEQUENCE commit_failure_witness; CREATE FUNCTION fail_during_commit() RETURNS trigger LANGUAGE plpgsql AS $$BEGIN PERFORM nextval('commit_failure_witness'); RAISE EXCEPTION 'fixture deferred failure' USING ERRCODE='23503'; END$$; CREATE CONSTRAINT TRIGGER fail_during_commit AFTER INSERT ON documents DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION fail_during_commit()")
                    .execute(&pool).await.unwrap();
            }
            armed.store(true,Ordering::SeqCst);
            let payload=json!({"title":"Wire document","content_markdown":"Original command result"});
            let (status,body)=call(&router,"/api/v1/spaces/WIRE/documents",Some(token),Some("wire-key"),payload.clone()).await;
            assert!(lost.load(Ordering::SeqCst),"the actual COMMIT outcome reply must be lost");
            if rollback || refuse {assert!(status.is_server_error(),"{status}: {body}");}
            else {assert_eq!(status,StatusCode::CREATED,"confirmed original COMMIT: {body}");}
            assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM documents WHERE title='Wire document'").fetch_one(&pool).await.unwrap(),if rollback{0}else{1});
            if rollback {
                assert_eq!(sqlx::query_scalar::<_,i64>("SELECT last_value FROM commit_failure_witness").fetch_one(&pool).await.unwrap(),1);
                sqlx::query("DROP TRIGGER fail_during_commit ON documents").execute(&pool).await.unwrap();
            }
            let state:Option<String>=sqlx::query_scalar("SELECT state FROM idempotency_records WHERE idempotency_key='wire-key'").fetch_optional(&pool).await.unwrap();
            if !rollback {assert_eq!(state.as_deref(),Some("completed"));}
            else if refuse {assert_eq!(state.as_deref(),Some("processing"));}
            else {assert!(state.is_none(),"confirmed rollback releases only its original reservation");}
            refusal.store(false,Ordering::SeqCst);
            let before=snapshot(&pool).await;
            let (status,replayed)=call(&router,"/api/v1/spaces/WIRE/documents",Some(token),Some("wire-key"),payload.clone()).await;
            if rollback&&refuse {
                assert_eq!(status,StatusCode::CONFLICT,"an unresolved reservation is not silently reexecuted");
                assert_eq!(snapshot(&pool).await,before);
            } else {
                assert_eq!(status,StatusCode::CREATED,"{replayed}");
                if !rollback {assert_eq!(snapshot(&pool).await,before,"lost reply retry must not mutate content/audit");}
                assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM documents WHERE title='Wire document'").fetch_one(&pool).await.unwrap(),1);
                let before=snapshot(&pool).await;
                let (status,again)=call(&router,"/api/v1/spaces/WIRE/documents",Some(token),Some("wire-key"),payload).await;
                assert_eq!(status,StatusCode::CREATED);assert_eq!(again,replayed);assert_eq!(snapshot(&pool).await,before);
            }
            pool.close().await;
        }).await;
        fixture.close().await;
        result.unwrap();
    }
}
#[tokio::test]
#[ignore = "manual: current standalone user and resource authorization on replay"]
async fn cached_response_does_not_bypass_revoked_resource_permission_or_disabled_identity() {
    let fixture = Fixture::new().await;
    let router = fixture.router.clone();
    let pool = fixture.control.clone();
    let result=tokio::spawn(async move{
        let (status,login)=call(&router,"/api/v1/auth/login",None,None,json!({"email":"result@example.test","password":"Synthetic result fixture 42!"})).await;
        assert_eq!(status,StatusCode::OK);let admin=login["access_token"].as_str().unwrap();
        let (status,user)=call(&router,"/api/v1/users",Some(admin),None,json!({"email":"editor@example.test","username":"editor","password":"Synthetic editor fixture 42!","display_name":"Editor","role":"user"})).await;
        assert_eq!(status,StatusCode::CREATED);let user:Uuid=user["id"].as_str().unwrap().parse().unwrap();
        let (status,space)=call(&router,"/api/v1/spaces",Some(admin),None,json!({"key":"ACCESS","name":"Replay access"})).await;
        assert_eq!(status,StatusCode::CREATED);let space:Uuid=space["id"].as_str().unwrap().parse().unwrap();
        let (status,member)=send(&router,"PUT",&format!("/api/v1/spaces/ACCESS/members/{user}"),admin,None,"application/json",json!({"role":"editor"}).to_string().into_bytes()).await;
        assert_eq!(status,StatusCode::OK,"{member}");
        let (status,login)=call(&router,"/api/v1/auth/login",None,None,json!({"email":"editor@example.test","password":"Synthetic editor fixture 42!"})).await;
        assert_eq!(status,StatusCode::OK);let token=login["access_token"].as_str().unwrap();
        let payload=json!({"title":"Private draft","content_markdown":"Private result"});
        let (status,first)=call(&router,"/api/v1/spaces/ACCESS/documents",Some(token),Some("permission-key"),payload.clone()).await;
        assert_eq!(status,StatusCode::CREATED,"{first}");
        sqlx::query("UPDATE space_members SET role='viewer' WHERE space_id=$1 AND user_id=$2").bind(space).bind(user).execute(&pool).await.unwrap();
        let (status,denied)=call(&router,"/api/v1/spaces/ACCESS/documents",Some(token),Some("permission-key"),payload.clone()).await;
        assert_eq!(status,StatusCode::FORBIDDEN,"fresh resource authorization: {denied}");
        sqlx::query("UPDATE space_members SET role='editor' WHERE space_id=$1 AND user_id=$2").bind(space).bind(user).execute(&pool).await.unwrap();
        let (status,restored)=call(&router,"/api/v1/spaces/ACCESS/documents",Some(token),Some("permission-key"),payload.clone()).await;
        assert_eq!(status,StatusCode::CREATED);assert_eq!(restored,first);
        sqlx::query("UPDATE users SET is_active=false WHERE id=$1").bind(user).execute(&pool).await.unwrap();
        let (status,_)=call(&router,"/api/v1/spaces/ACCESS/documents",Some(token),Some("permission-key"),payload).await;
        assert_eq!(status,StatusCode::UNAUTHORIZED);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM documents WHERE title='Private draft'").fetch_one(&pool).await.unwrap(),1);
        pool.close().await;
    }).await;
    fixture.close().await;
    result.unwrap();
}

#[tokio::test]
#[ignore = "manual: concurrent protected command writers on actual PostgreSQL"]
async fn concurrent_same_keys_execute_once_and_independent_keys_remain_independent() {
    let fixture = Fixture::new().await;
    let router = fixture.router.clone();
    let pool = fixture.control.clone();
    let result=tokio::spawn(async move{
        let (status,login)=call(&router,"/api/v1/auth/login",None,None,json!({"email":"result@example.test","password":"Synthetic result fixture 42!"})).await;
        assert_eq!(status,StatusCode::OK);let token=login["access_token"].as_str().unwrap().to_owned();
        let (status,_)=call(&router,"/api/v1/spaces",Some(&token),None,json!({"key":"RACE","name":"Concurrent commands"})).await;
        assert_eq!(status,StatusCode::CREATED);
        sqlx::raw_sql("CREATE FUNCTION delay_result() RETURNS trigger LANGUAGE plpgsql AS $$BEGIN PERFORM pg_sleep(0.1);RETURN NEW;END$$;CREATE TRIGGER delay_result BEFORE UPDATE ON idempotency_records FOR EACH ROW WHEN(NEW.state='completed') EXECUTE FUNCTION delay_result()")
            .execute(&pool).await.unwrap();
        let mut writers=tokio::task::JoinSet::new();
        for _ in 0..16 {
            let (router,token)=(router.clone(),token.clone());
            writers.spawn(async move{call(&router,"/api/v1/spaces/RACE/documents",Some(&token),Some("same-key"),json!({"title":"One original"})).await});
        }
        let mut first=None;
        while let Some(result)=writers.join_next().await {
            let (status,response)=result.unwrap();
            assert!([StatusCode::CREATED,StatusCode::CONFLICT].contains(&status),"{status}: {response}");
            if status==StatusCode::CREATED {
                if let Some(first)=first.as_ref(){assert_eq!(&response,first);}else{first=Some(response);}
            }
        }
        assert!(first.is_some());
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM documents WHERE title='One original'").fetch_one(&pool).await.unwrap(),1);
        let (status,replayed)=call(&router,"/api/v1/spaces/RACE/documents",Some(&token),Some("same-key"),json!({"title":"One original"})).await;
        assert_eq!(status,StatusCode::CREATED);assert_eq!(Some(replayed),first);
        for index in 0..16 {
            let (router,token)=(router.clone(),token.clone());
            writers.spawn(async move{call(&router,"/api/v1/spaces/RACE/documents",Some(&token),Some(&format!("independent-{index}")),json!({"title":format!("Independent {index}")})).await});
        }
        while let Some(result)=writers.join_next().await {
            let (status,response)=result.unwrap();assert_eq!(status,StatusCode::CREATED,"{response}");
        }
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM documents").fetch_one(&pool).await.unwrap(),17);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM audit_log WHERE action='document.create'").fetch_one(&pool).await.unwrap(),17);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM idempotency_records WHERE state='completed'").fetch_one(&pool).await.unwrap(),17);
        pool.close().await;
    }).await;
    fixture.close().await;
    result.unwrap();
}
