//! Real Wiki router/SQLx; only the Central Auth transport is a controlled fixture.
use axum::{
    Json, Router,
    body::{Body, to_bytes},
    http::{HeaderMap, Method, Request, StatusCode},
    routing::{get, post},
};
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tower::ServiceExt;
use uuid::Uuid;

const SUBJECT: &str = "00000000-0000-4000-8000-000000000931";
const OWNER: &str = "00000000-0000-4000-8000-000000000932";

async fn request(app: &Router, token: &str, method: Method, path: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}

#[test]
#[ignore = "requires fresh owned WIKI_TEST_DATABASE_URL ending _analyst_test"]
fn machine_reader_is_read_only_project_limited_revocable_and_never_jits() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let issuer = format!("http://{}", listener.local_addr().unwrap());
    // This binary has one synchronous test. Configure before starting any runtime.
    unsafe {
        std::env::set_var("WIKI_AUTH__CENTRAL_JWKS_URI", format!("{issuer}/oidc/jwks"));
        std::env::set_var("WIKI_AUTH__CENTRAL_ISSUER", &issuer);
        std::env::set_var("WIKI_READERS__SUBJECTS", SUBJECT);
        std::env::set_var("WIKI_SPACES__ADMISSION_URL", format!("{issuer}/admission"));
        std::env::set_var(
            "WIKI_SPACES__ADMISSION_TOKEN",
            "synthetic-reader-service-token",
        );
    }
    tokio::runtime::Runtime::new().unwrap().block_on(async move {
        let live = Arc::new(AtomicBool::new(true));
        let active = live.clone();
        let auth = Router::new().route("/oidc/jwks", get(|| async { Json(json!({"keys":[]})) }))
            .route("/auth/tokens/introspect", get(move |headers: HeaderMap| {
                let active = active.clone();
                async move {
                    let token = headers.get("authorization").and_then(|v| v.to_str().ok()).unwrap_or("");
                    if !active.load(Ordering::SeqCst) { return (StatusCode::UNAUTHORIZED, Json(json!({}))); }
                    let (subject, role, scopes) = match token {
                        "Bearer sdlc_pat_analyst-owner" => (OWNER.to_owned(), "admin", vec!["wiki:read", "wiki:write"]),
                        "Bearer sdlc_pat_analyst-reader" => (SUBJECT.to_owned(), "service_account", vec!["wiki:read"]),
                        "Bearer sdlc_pat_analyst-unregistered" => (Uuid::new_v4().to_string(), "service_account", vec!["wiki:read"]),
                        "Bearer sdlc_pat_analyst-writer" => (SUBJECT.to_owned(), "service_account", vec!["wiki:read", "wiki:write"]),
                        "Bearer sdlc_pat_analyst-human" => (SUBJECT.to_owned(), "user", vec!["wiki:read"]),
                        _ => return (StatusCode::UNAUTHORIZED, Json(json!({}))),
                    };
                    let now = chrono::Utc::now();
                    let email = if subject == OWNER { "owner@example.test" } else { "reader@example.test" };
                    (StatusCode::OK, Json(json!({"sub":subject,"role":role,"subject_role":role,
                        "email":email,"display_name":"Synthetic reader","scopes":scopes,
                        "observed_at":now,"expires_at":now+chrono::Duration::seconds(60)})))
                }
            }))
            .route("/admission", post(|headers: HeaderMap, Json(query): Json<Value>| async move {
                assert_eq!(headers["authorization"], "Bearer synthetic-reader-service-token");
                assert!([SUBJECT, OWNER].contains(&query["identity"]["subject"].as_str().unwrap()));
                let now = chrono::Utc::now().timestamp();
                Json(json!({"schema_version":1,"identity":query["identity"],"identity_active":true,
                    "policy_revision":1,"policy":{"mode":"all"},"validity":{"observed_at":now,"valid_until":now+60},
                    "spaces":query["spaces"].as_array().unwrap().iter().map(|space| json!({"space":space,"revision":1,"state":"active"})).collect::<Vec<_>>()}))
            }));
        let server = tokio::spawn(async move { axum::serve(tokio::net::TcpListener::from_std(listener).unwrap(),auth).await.unwrap(); });
        let url = std::env::var("WIKI_TEST_DATABASE_URL").unwrap();
        let pool = PgPool::connect(&url).await.unwrap();
        let database: String = sqlx::query_scalar("SELECT current_database()").fetch_one(&pool).await.unwrap();
        assert!(database.ends_with("_analyst_test"));
        assert!(sqlx::query_scalar::<_,bool>("SELECT to_regclass('public._sqlx_migrations') IS NULL").fetch_one(&pool).await.unwrap());
        let storage_dir = std::env::temp_dir().join(format!("wiki-analyst-{}",Uuid::new_v4()));
        let mut cfg = shared::AppConfig::default();
        cfg.database.url = url;
        cfg.storage.dir = storage_dir.to_string_lossy().into_owned();
        cfg.server.general_rate_burst = 1000;
        let cfg=Arc::new(cfg);
        let storage=Arc::new(infra::LocalWikiAttachmentStorage::new(&storage_dir));
        let (backend, settings)=infra::connect_postgres_wiki_backend(&cfg, storage).await.unwrap();
        let ctx=Arc::new(app::WikiAppContext::new(cfg.clone()));
        let app=api::router_with_wiki(ctx.clone(),api::routes::wiki::WikiBackend::persistent(backend,settings)).with_state(ctx);
        let token="sdlc_pat_analyst-reader";
        // Even a registered processor cannot create its local profile by reading.
        assert_eq!(request(&app,token,Method::GET,"/api/v1/spaces").await.0,StatusCode::UNAUTHORIZED);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM users").fetch_one(&pool).await.unwrap(),0);
        for token in ["sdlc_pat_analyst-unregistered","sdlc_pat_analyst-writer","sdlc_pat_analyst-human"] {
            assert!(infra::prepare_registered_reader(&cfg, token, SUBJECT).await.is_err());
        }
        assert!(infra::prepare_registered_reader(&cfg, token, OWNER).await.is_err());
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM users").fetch_one(&pool).await.unwrap(),0);
        let user=infra::prepare_registered_reader(&cfg, token, SUBJECT).await.unwrap();
        assert_eq!(infra::prepare_registered_reader(&cfg, token, SUBJECT).await.unwrap(),user);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM space_members").fetch_one(&pool).await.unwrap(),0);
        assert_eq!(sqlx::query_scalar::<_,String>("SELECT global_role FROM users WHERE id=$1").bind(user).fetch_one(&pool).await.unwrap(),"user");
        // Historical local admin state must never elevate a registered machine.
        sqlx::query("UPDATE users SET global_role='admin' WHERE id=$1")
            .bind(user).execute(&pool).await.unwrap();
        let owner=Uuid::new_v4();
        sqlx::query("INSERT INTO users(id,email,username,display_name,password_hash,central_sub,global_role) VALUES($1,'owner@example.test','owner','Synthetic owner','!',$2,'user')")
            .bind(owner).bind(OWNER).execute(&pool).await.unwrap();
        let mut documents=Vec::new();
        for key in ["ALLOWED","FOREIGN"] {
            let space=Uuid::new_v4();let doc=Uuid::new_v4();let revision=Uuid::new_v4();
            sqlx::query("INSERT INTO spaces(id,key,name,owner_id) VALUES($1,$2,$2,$3)").bind(space).bind(key).bind(owner).execute(&pool).await.unwrap();
            if key=="ALLOWED" { sqlx::query("INSERT INTO space_members(space_id,user_id,role) VALUES($1,$2,'editor')").bind(space).bind(user).execute(&pool).await.unwrap(); }
            sqlx::query("INSERT INTO documents(id,space_id,slug,title,owner_id) VALUES($1,$2,'source','Source',$3)").bind(doc).bind(space).bind(user).execute(&pool).await.unwrap();
            sqlx::query("INSERT INTO document_revisions(id,document_id,version,title,content_markdown,content_html,content_text,content_checksum,author_id) VALUES($1,$2,1,'Source','Published needle','Published needle','Published needle','test',$3)")
                .bind(revision).bind(doc).bind(user).execute(&pool).await.unwrap();
            sqlx::query("UPDATE documents SET current_revision_id=$2,status='published' WHERE id=$1").bind(doc).bind(revision).execute(&pool).await.unwrap();
            sqlx::query("INSERT INTO document_drafts(document_id,author_id,content_markdown) VALUES($1,$2,'PRIVATE DRAFT needle')").bind(doc).bind(user).execute(&pool).await.unwrap();
            documents.push((space,doc));
        }
        let (status,spaces)=request(&app,token,Method::GET,"/api/v1/spaces").await;
        assert_eq!(status,StatusCode::OK);assert!(spaces.to_string().contains("ALLOWED"));assert!(!spaces.to_string().contains("FOREIGN"));
        let (status,doc)=request(&app,token,Method::GET,&format!("/api/v1/documents/{}",documents[0].1)).await;
        assert_eq!(status,StatusCode::OK);assert!(doc.to_string().contains("Published needle"));assert!(!doc.to_string().contains("PRIVATE DRAFT"));assert_eq!(doc["can_edit"],false);
        assert_eq!(request(&app,token,Method::GET,&format!("/api/v1/documents/{}",documents[1].1)).await.0,StatusCode::FORBIDDEN);
        let (status,search)=request(&app,token,Method::GET,"/api/v1/search?q=needle").await;
        assert_eq!(status,StatusCode::OK);assert!(!search.to_string().contains("FOREIGN"));assert!(!search.to_string().contains("PRIVATE DRAFT"));
        for (method,path) in [(Method::POST,"/api/v1/spaces".into()),(Method::GET,"/api/v1/users".into()),
            (Method::PUT,format!("/api/v1/documents/{}/draft",documents[0].1)),(Method::GET,"/api/v1/spaces/ALLOWED/members".into())] {
            assert_eq!(request(&app,token,method,&path).await.0,StatusCode::FORBIDDEN);
        }
        for token in ["sdlc_pat_analyst-unregistered","sdlc_pat_analyst-writer","sdlc_pat_analyst-human"] {
            assert_eq!(request(&app,token,Method::GET,"/api/v1/spaces").await.0,StatusCode::FORBIDDEN);
        }
        // The common admission's All policy cannot replace explicit local membership.
        let registry=Uuid::new_v4();
        for (space,_) in &documents {
            sqlx::query("INSERT INTO wiki_space_catalog(registry_instance_id,space_id,local_space_id,admin_revision,state,operation_id) VALUES($1,$2,$3,1,'active',$4)")
                .bind(registry).bind(Uuid::new_v4()).bind(space).bind(Uuid::new_v4()).execute(&pool).await.unwrap();
        }
        sqlx::query("UPDATE wiki_space_admission_state SET active=true,cohort_revision='synthetic-analyst-reader-fixture',role_migration_confirmed=true WHERE singleton").execute(&pool).await.unwrap();
        let (status,spaces)=request(&app,token,Method::GET,"/api/v1/spaces").await;
        assert_eq!(status,StatusCode::OK);assert!(spaces.to_string().contains("ALLOWED"));assert!(!spaces.to_string().contains("FOREIGN"));
        let (status,doc)=request(&app,token,Method::GET,&format!("/api/v1/documents/{}",documents[0].1)).await;
        assert_eq!(status,StatusCode::OK);assert!(!doc.to_string().contains("PRIVATE DRAFT"));assert_eq!(doc["can_edit"],false);
        assert_eq!(request(&app,token,Method::GET,&format!("/api/v1/documents/{}",documents[1].1)).await.0,StatusCode::FORBIDDEN);
        // A genuine human administrator revokes through the normal protected API.
        assert_eq!(request(&app,"sdlc_pat_analyst-owner",Method::DELETE,
            &format!("/api/v1/spaces/ALLOWED/members/{user}")).await.0,StatusCode::NO_CONTENT);
        assert_eq!(request(&app,token,Method::GET,&format!("/api/v1/documents/{}",documents[0].1)).await.0,StatusCode::FORBIDDEN);
        sqlx::query("UPDATE users SET is_active=false WHERE id=$1").bind(user).execute(&pool).await.unwrap();
        assert!(infra::prepare_registered_reader(&cfg, token, SUBJECT).await.is_err());
        assert!(!sqlx::query_scalar::<_,bool>("SELECT is_active FROM users WHERE id=$1").bind(user).fetch_one(&pool).await.unwrap());
        live.store(false,Ordering::SeqCst);
        assert_eq!(request(&app,token,Method::GET,"/api/v1/spaces").await.0,StatusCode::UNAUTHORIZED);
        assert_eq!(sqlx::query("SELECT global_role FROM users WHERE id=$1").bind(user).fetch_one(&pool).await.unwrap().get::<String,_>("global_role"),"admin");
        server.abort();pool.close().await;
    });
}
