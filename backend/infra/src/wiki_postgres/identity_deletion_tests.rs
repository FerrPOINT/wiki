//! Explicit real PostgreSQL gate; a schema is owned and cleaned even after a panic.
use super::identity_deletion::{IdentityDeletionStore, OwnerError};
use pdlc_shared::{identity_lifecycle::*, space_access::CentralIdentity};
use sqlx::{Connection, PgPool};
use std::{future::Future, path::Path};
use uuid::Uuid;
const ISSUER: &str = "https://auth.example.test";
fn window() -> shared::identity_deletion::IdentityDeletionWindow {
    shared::identity_deletion::IdentityDeletionWindow::restricted(
        std::time::Instant::now() + std::time::Duration::from_secs(60),
        chrono::Utc::now().timestamp() + 60,
    )
}
fn expired_window() -> shared::identity_deletion::IdentityDeletionWindow {
    shared::identity_deletion::IdentityDeletionWindow::restricted(
        std::time::Instant::now() + std::time::Duration::from_secs(60),
        chrono::Utc::now().timestamp() - 1,
    )
}

struct Database {
    pool: PgPool,
    admin: sqlx::PgConnection,
    schema: String,
}
impl Database {
    async fn new(predecessor: bool) -> Self {
        let raw = std::env::var("WIKI_TEST_DATABASE_URL")
            .expect("explicit disposable WIKI_TEST_DATABASE_URL required");
        let mut url = reqwest::Url::parse(&raw).unwrap();
        assert!(matches!(url.scheme(), "postgres" | "postgresql"));
        assert!(
            url.path()
                .strip_prefix('/')
                .is_some_and(|name| name.ends_with("_test") && !name.contains('/'))
        );
        assert!(
            !url.query_pairs()
                .any(|(key, _)| key == "options" || key.starts_with("options[") || key == "dbname")
        );
        let mut admin = sqlx::PgConnection::connect(&raw).await.unwrap();
        let schema = format!("wiki_identity_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&mut admin)
            .await
            .unwrap();
        url.query_pairs_mut()
            .append_pair("options", &format!("-c search_path={schema}"));
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(8)
            .connect(url.as_str())
            .await
            .unwrap();
        let all = sqlx::migrate::Migrator::new(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("migrations"),
        )
        .await
        .unwrap();
        if predecessor {
            let mut old = sqlx::migrate::Migrator::DEFAULT;
            old.migrations = std::borrow::Cow::Owned(
                all.iter()
                    .filter(|migration| migration.version < 202610220003)
                    .cloned()
                    .collect(),
            );
            old.run(&pool).await.unwrap();
        } else {
            all.run(&pool).await.unwrap();
        }
        Self {
            pool,
            admin,
            schema,
        }
    }
    async fn close(mut self) {
        self.pool.close().await;
        sqlx::query(&format!("DROP SCHEMA {} CASCADE", self.schema))
            .execute(&mut self.admin)
            .await
            .unwrap();
        self.admin.close().await.unwrap();
    }
}
async fn isolated<F, Fut>(old: bool, test: F)
where
    F: FnOnce(PgPool) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let db = Database::new(old).await;
    let result = tokio::spawn(test(db.pool.clone())).await;
    db.close().await;
    assert!(
        result.is_ok(),
        "identity owner qualification failed; owned schema cleaned"
    );
}
fn request(subject: Uuid) -> DeletionRequest {
    DeletionRequest {
        schema_version: 1,
        operation_id: Uuid::new_v4(),
        identity: CentralIdentity {
            issuer: ISSUER.into(),
            subject: subject.to_string(),
        },
        inventory_revision: Uuid::new_v4(),
    }
}
async fn insert_user(pool: &PgPool, subject: Option<Uuid>) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO users(id,email,username,display_name,password_hash,central_sub) VALUES($1,$2,$3,'Synthetic identity','!', $4)").bind(id).bind(format!("{id}@example.test")).bind(format!("user-{id}")).bind(subject.map(|id|id.to_string())).execute(pool).await.unwrap();
    id
}
async fn blocked_by(pool: &PgPool, pid: i32) {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let blocked: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE $1=ANY(pg_blocking_pids(pid)))",
        )
        .bind(pid)
        .fetch_one(pool)
        .await
        .unwrap();
        if blocked {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "missing lock barrier"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}
#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL"]
async fn populated_identity_aliases_survive_profile_deletion_without_email_guessing() {
    isolated(true,|pool|async move{
 let central=Uuid::new_v4();let local=insert_user(&pool,Some(central)).await;let unrelated=insert_user(&pool,None).await;
 let before:serde_json::Value=sqlx::query_scalar("SELECT to_jsonb(users) FROM users WHERE id=$1").bind(local).fetch_one(&pool).await.unwrap();
 let all=sqlx::migrate::Migrator::new(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("migrations")).await.unwrap();all.run(&pool).await.unwrap();
 let after:serde_json::Value=sqlx::query_scalar("SELECT to_jsonb(users) FROM users WHERE id=$1").bind(local).fetch_one(&pool).await.unwrap();assert_eq!(before,after);
 let store=IdentityDeletionStore::new(pool.clone());let command=request(central);assert!(matches!(store.check(&command,&window()).await,Err(OwnerError::Unbound)));
 store.bind_legacy_issuer(ISSUER).await.unwrap();assert!(store.check(&command,&window()).await.unwrap().reference_count>=1);
 assert!(store.bind_legacy_issuer("https://other.example.test").await.is_err());assert!(sqlx::query("UPDATE wiki_identity_legacy_issuer SET issuer='https://other.example.test'").execute(&pool).await.is_err());
 sqlx::query("UPDATE users SET username='renamed',email='renamed@example.test' WHERE id=$1").bind(local).execute(&pool).await.unwrap();
 assert!(sqlx::query("UPDATE users SET central_sub=$2 WHERE id=$1").bind(local).bind(Uuid::new_v4().to_string()).execute(&pool).await.is_err());
 sqlx::query("DELETE FROM users WHERE id=$1").bind(local).execute(&pool).await.unwrap();
 sqlx::query("INSERT INTO audit_log(id,action,entity_type,entity_id,request_id) VALUES($1,'user.update','user',$2,'synthetic')").bind(Uuid::new_v4()).bind(local).execute(&pool).await.unwrap();
 assert_eq!(store.check(&command,&window()).await.unwrap().reference_count,2);
 let receipt=store.freeze(&command,&window()).await.unwrap();assert_eq!(receipt.reference_count,2);
 assert!(sqlx::query("INSERT INTO audit_log(id,actor_id,action,entity_type,entity_id,request_id) VALUES($1,$2,'user.update','user',$3,'synthetic')").bind(Uuid::new_v4()).bind(unrelated).bind(local).execute(&pool).await.is_err());
 let alias:(String,String)=sqlx::query_as("SELECT issuer,subject FROM wiki_identity_aliases WHERE local_user_id=$1").bind(local).fetch_one(&pool).await.unwrap();assert_eq!(alias,(ISSUER.into(),central.to_string()));
}).await;
}
#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL"]
async fn empty_owner_receipts_replay_and_release_fences_late_freeze() {
    isolated(false, |pool| async move {
        let store = IdentityDeletionStore::new(pool);
        store.bind_legacy_issuer(ISSUER).await.unwrap();
        let command = request(Uuid::new_v4());
        assert_eq!(
            store
                .check(&command, &window())
                .await
                .unwrap()
                .reference_count,
            0
        );
        let first = store.freeze(&command, &window()).await.unwrap();
        assert_eq!(first.reference_count, 0);
        assert_eq!(first, store.freeze(&command, &window()).await.unwrap());
        let mut other = command.clone();
        other.inventory_revision = Uuid::new_v4();
        assert!(matches!(
            store.freeze(&other, &window()).await,
            Err(OwnerError::Conflict)
        ));
        let mut competing = command.clone();
        competing.operation_id = Uuid::new_v4();
        assert!(matches!(
            store.freeze(&competing, &window()).await,
            Err(OwnerError::Conflict)
        ));
        store.release(&competing, &window()).await.unwrap();
        assert_eq!(first, store.freeze(&command, &window()).await.unwrap());
        store.release(&command, &window()).await.unwrap();
        assert!(matches!(
            store.freeze(&command, &window()).await,
            Err(OwnerError::Conflict)
        ));
        let late = request(Uuid::new_v4());
        assert_eq!(
            store.release(&late, &window()).await.unwrap(),
            store.release(&late, &window()).await.unwrap()
        );
        assert!(matches!(
            store.freeze(&late, &window()).await,
            Err(OwnerError::Conflict)
        ));
        assert!(matches!(
            store
                .freeze(&request(Uuid::new_v4()), &expired_window())
                .await,
            Err(OwnerError::Expired)
        ));
    })
    .await;
}
#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL"]
async fn freeze_drains_old_writer_and_repeatable_read_cannot_use_an_obsolete_epoch() {
    isolated(false,|pool|async move{
 let central=Uuid::new_v4();let local=insert_user(&pool,Some(central)).await;let store=IdentityDeletionStore::new(pool.clone());store.bind_legacy_issuer(ISSUER).await.unwrap();
 let mut writer=pool.begin().await.unwrap();let pid:i32=sqlx::query_scalar("SELECT pg_backend_pid()").fetch_one(&mut *writer).await.unwrap();
 sqlx::query("INSERT INTO audit_log(id,actor_id,action,entity_type,entity_id,request_id) VALUES($1,$2,'synthetic','document',$3,'synthetic')").bind(Uuid::new_v4()).bind(local).bind(Uuid::new_v4()).execute(&mut *writer).await.unwrap();
 let frozen=store.clone();let command=request(central);let task=tokio::spawn(async move{frozen.freeze(&command,&window()).await});blocked_by(&pool,pid).await;writer.commit().await.unwrap();assert_eq!(task.await.unwrap().unwrap().reference_count,3);
 let mut old=pool.begin().await.unwrap();sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ").execute(&mut *old).await.unwrap();sqlx::query("SELECT count(*) FROM users").execute(&mut *old).await.unwrap();
 let central=Uuid::new_v4();store.freeze(&request(central),&window()).await.unwrap();
 let result=sqlx::query("INSERT INTO users(id,email,username,display_name,password_hash,central_sub) VALUES($1,$2,$3,'Synthetic','!',$4)").bind(Uuid::new_v4()).bind(format!("{central}@example.test")).bind(format!("late-{central}")).bind(central.to_string()).execute(&mut *old).await;
 assert_eq!(result.unwrap_err().as_database_error().unwrap().code().as_deref(),Some("40001"));old.rollback().await.unwrap();
}).await;
}
#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL"]
async fn actual_orphan_history_denies_zero_but_arbitrary_text_is_not_an_identity_mapping() {
    isolated(true,|pool|async move{
 let orphan=Uuid::new_v4();
 sqlx::query("INSERT INTO audit_log(id,action,entity_type,entity_id,request_id) VALUES($1,'user.update','user',$2,'legacy-synthetic')").bind(Uuid::new_v4()).bind(orphan).execute(&pool).await.unwrap();
 let all=sqlx::migrate::Migrator::new(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("migrations")).await.unwrap();all.run(&pool).await.unwrap();let store=IdentityDeletionStore::new(pool.clone());store.bind_legacy_issuer(ISSUER).await.unwrap();
 assert!(matches!(store.check(&request(Uuid::new_v4()),&window()).await,Err(OwnerError::Unbound)));
 assert!(sqlx::query("INSERT INTO wiki_identity_aliases(local_user_id,subject) VALUES($1,$2)").bind(orphan).bind(Uuid::new_v4().to_string()).execute(&pool).await.is_err());
 assert!(matches!(store.check(&request(Uuid::new_v4()),&window()).await,Err(OwnerError::Unbound)),"unbound alias cannot turn orphan history into zero");
 assert!(sqlx::query("INSERT INTO audit_log(id,action,entity_type,entity_id,request_id) VALUES($1,'user.update','user',$2,'synthetic')").bind(Uuid::new_v4()).bind(orphan).execute(&pool).await.is_err());

 let refs:i64=sqlx::query_scalar("SELECT count(*) FROM wiki_identity_json_refs('{\"content_markdown\":\"arbitrary personal text\",\"email\":\"somebody@example.test\"}'::jsonb,$1)").bind(ISSUER).fetch_one(&pool).await.unwrap();assert_eq!(refs,0);
}).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL"]
async fn every_owned_reference_table_rejects_a_raw_sql_writer_after_freeze() {
    isolated(false,|pool|async move{
 let central=Uuid::new_v4();let target=insert_user(&pool,Some(central)).await;let owner=insert_user(&pool,None).await;
 let space=Uuid::new_v4();let doc=Uuid::new_v4();let rev=Uuid::new_v4();let task=Uuid::new_v4();let phase=Uuid::new_v4();let registry=Uuid::new_v4();let canonical=Uuid::new_v4();
 sqlx::raw_sql(&format!("INSERT INTO spaces(id,key,name,owner_id) VALUES('{space}','QA01','Synthetic space','{owner}');
 INSERT INTO documents(id,space_id,slug,title,owner_id) VALUES('{doc}','{space}','synthetic-document','Synthetic','{owner}');
 INSERT INTO document_revisions(id,document_id,version,title,content_markdown,content_html,content_text,content_checksum,author_id) VALUES('{rev}','{doc}',1,'Synthetic','Synthetic','Synthetic','Synthetic','synthetic','{owner}');
 INSERT INTO task_dossiers(id,space_id,task_key) VALUES('{task}','{space}','TASK-1');
 INSERT INTO phase_dossiers(id,space_id,phase_key) VALUES('{phase}','{space}','plan');
 INSERT INTO wiki_space_catalog(registry_instance_id,space_id,local_space_id,state) VALUES('{registry}','{canonical}','{space}','active');")).execute(&pool).await.unwrap();
 let store=IdentityDeletionStore::new(pool.clone());store.bind_legacy_issuer(ISSUER).await.unwrap();assert_eq!(store.freeze(&request(central),&window()).await.unwrap().reference_count,2);
 let writes=vec![
 format!("INSERT INTO users(id,email,username,display_name,password_hash,central_sub) VALUES('{}','blocked@example.test','blocked','Synthetic','!','{central}')",Uuid::new_v4()),
 format!("INSERT INTO spaces(id,key,name,owner_id) VALUES('{}','QA02','Synthetic','{target}')",Uuid::new_v4()),
 format!("INSERT INTO space_members(space_id,user_id,role) VALUES('{space}','{target}','viewer')"),
 format!("INSERT INTO documents(id,space_id,slug,title,owner_id) VALUES('{}','{space}','blocked-document','Synthetic','{target}')",Uuid::new_v4()),
 format!("INSERT INTO document_revisions(id,document_id,version,title,content_markdown,content_html,content_text,content_checksum,author_id) VALUES('{}','{doc}',2,'Synthetic','Synthetic','Synthetic','Synthetic','synthetic','{target}')",Uuid::new_v4()),
 format!("INSERT INTO document_drafts(document_id,author_id) VALUES('{doc}','{target}')"),
 format!("INSERT INTO document_task_links(space_id,document_id,task_dossier_id,created_by) VALUES('{space}','{doc}','{task}','{target}')"),
 format!("INSERT INTO document_phase_links(space_id,document_id,phase_dossier_id,created_by) VALUES('{space}','{doc}','{phase}','{target}')"),
 format!("INSERT INTO attachments(id,file_name,content_type,size_bytes,storage_key,checksum,uploaded_by) VALUES('{}','synthetic.txt','text/plain',1,'synthetic','synthetic','{target}')",Uuid::new_v4()),
 format!("INSERT INTO evidence_items(id,space_id,document_id,evidence_type,title,url,created_by) VALUES('{}','{space}','{doc}','external_url','Synthetic','https://example.test','{target}')",Uuid::new_v4()),
 format!("INSERT INTO audit_log(id,actor_id,action,entity_type,entity_id,request_id) VALUES('{}','{target}','synthetic','document','{doc}','synthetic')",Uuid::new_v4()),
 format!("INSERT INTO auth_sessions(id,user_id,access_token_hash,refresh_token_hash,expires_at,refresh_expires_at) VALUES('{}','{target}','synthetic-access','synthetic-refresh',now()+interval '1 hour',now()+interval '1 day')",Uuid::new_v4()),
 format!("INSERT INTO idempotency_records(id,actor_id,idempotency_key,method,path,request_hash,state) VALUES('{}','{target}','synthetic','POST','/synthetic','synthetic','processing')",Uuid::new_v4()),
 format!("INSERT INTO managed_task_document_links(id,space_id,dossier_id,document_id,revision_id,created_by) VALUES('{}','{space}','{task}','{doc}','{rev}','{target}')",Uuid::new_v4()),
 format!("INSERT INTO wiki_space_projects(space_id,registry_instance_id,namespace_id,created_by) VALUES('{space}','{}','{}','{target}')",Uuid::new_v4(),Uuid::new_v4()),
 format!("INSERT INTO wiki_confirmed_global_roles(user_id,role,granted_by) VALUES('{target}','admin','{owner}')"),
 format!("INSERT INTO wiki_confirmed_global_roles(user_id,role,granted_by) VALUES('{owner}','admin','{target}')"),
 format!("INSERT INTO wiki_namespace_bindings(resource_id,registry_instance_id,namespace_id,generation,state,command) VALUES('{space}','{registry}','{canonical}',1,'active','{{\"create_spec\":{{\"owner_subject\":\"{central}\"}}}}')"),
 format!("INSERT INTO wiki_project_space_projection_sets(tracker_instance_id,project_id,revision,operation_id,command) VALUES('{}','{}',1,'{}','{{\"actor_subject\":\"{central}\"}}')",Uuid::new_v4(),Uuid::new_v4(),Uuid::new_v4()),
 format!("INSERT INTO wiki_space_owner_operations(operation_id,registry_instance_id,space_id,revision,command,readback,owner_issuer,owner_subject) VALUES('{}','{registry}','{canonical}',1,'{{\"metadata\":{{\"responsible_subject\":\"{central}\"}}}}','{{}}','{ISSUER}','{}')",Uuid::new_v4(),Uuid::new_v4()),
 format!("INSERT INTO wiki_project_space_projection_operations(operation_id,tracker_instance_id,project_id,revision,command,readback,owner_issuer,owner_subject) VALUES('{}','{}','{}',1,'{{}}','{{}}','{ISSUER}','{central}')",Uuid::new_v4(),Uuid::new_v4(),Uuid::new_v4()),
 ];
 for (index,write) in writes.iter().enumerate(){let error=sqlx::query(write).execute(&pool).await.unwrap_err();assert_eq!(error.as_database_error().and_then(|error|error.constraint()),Some("wiki_identity_deletion_fenced"),"reference path {index} did not reject through the common identity fence");}
}).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL"]
async fn native_ids_are_not_central_subjects_and_proven_linking_cannot_cross_a_fence() {
    isolated(false,|pool|async move{
 let native=insert_user(&pool,None).await;let store=IdentityDeletionStore::new(pool.clone());store.bind_legacy_issuer(ISSUER).await.unwrap();
 sqlx::query("INSERT INTO audit_log(id,actor_id,action,entity_type,entity_id,request_id) VALUES($1,$2,'user.update','user',$2,'synthetic')").bind(Uuid::new_v4()).bind(native).execute(&pool).await.unwrap();
 assert_eq!(store.check(&request(native),&window()).await.unwrap().reference_count,0,"a native local UUID must not be guessed as a central subject");
 let central=Uuid::new_v4();let command=request(central);assert_eq!(store.freeze(&command,&window()).await.unwrap().reference_count,0);
 let result=sqlx::query("UPDATE users SET central_sub=$2 WHERE id=$1").bind(native).bind(central.to_string()).execute(&pool).await;
 assert_eq!(result.unwrap_err().as_database_error().and_then(|error|error.constraint()),Some("wiki_identity_deletion_fenced"));
 let local:(String,Option<String>,Option<String>)=sqlx::query_as("SELECT kind,issuer,subject FROM wiki_identity_aliases WHERE local_user_id=$1").bind(native).fetch_one(&pool).await.unwrap();assert_eq!(local,("local".into(),None,None));
 store.release(&command,&window()).await.unwrap();
 sqlx::query("UPDATE users SET central_sub=$2 WHERE id=$1").bind(native).bind(central.to_string()).execute(&pool).await.unwrap();
 let linked:(String,String,String)=sqlx::query_as("SELECT kind,issuer,subject FROM wiki_identity_aliases WHERE local_user_id=$1").bind(native).fetch_one(&pool).await.unwrap();assert_eq!(linked,("central".into(),ISSUER.into(),central.to_string()));
 assert_eq!(store.check(&request(central),&window()).await.unwrap().reference_count,3,"proven link must preserve and count preexisting local history");
 assert_eq!(store.check(&request(native),&window()).await.unwrap().reference_count,0);
 assert!(sqlx::query("UPDATE users SET central_sub=NULL WHERE id=$1").bind(native).execute(&pool).await.is_err());
}).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL"]
async fn deleted_native_profile_uuid_cannot_be_reused_to_reattribute_its_history() {
    isolated(false,|pool|async move{
 let native=insert_user(&pool,None).await;let store=IdentityDeletionStore::new(pool.clone());store.bind_legacy_issuer(ISSUER).await.unwrap();
 sqlx::query("INSERT INTO audit_log(id,action,entity_type,entity_id,request_id) VALUES($1,'user.update','user',$2,'synthetic')").bind(Uuid::new_v4()).bind(native).execute(&pool).await.unwrap();
 sqlx::query("DELETE FROM users WHERE id=$1").bind(native).execute(&pool).await.unwrap();
 let central=Uuid::new_v4();
 let error=sqlx::query("INSERT INTO users(id,email,username,display_name,password_hash,central_sub) VALUES($1,'reuse@example.test','synthetic-reuse','Synthetic','!',$2)").bind(native).bind(central.to_string()).execute(&pool).await.unwrap_err();
 assert_eq!(error.as_database_error().unwrap().code().as_deref(),Some("23514"));
 let alias:(String,Option<String>,Option<String>)=sqlx::query_as("SELECT kind,issuer,subject FROM wiki_identity_aliases WHERE local_user_id=$1").bind(native).fetch_one(&pool).await.unwrap();assert_eq!(alias,("local".into(),None,None));
 assert_eq!(store.check(&request(central),&window()).await.unwrap().reference_count,0,"native history must not move to a different central identity");
}).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL"]
async fn manual_alias_backfill_cannot_create_a_new_reference_after_freeze() {
    isolated(false, |pool| async move {
        let central = Uuid::new_v4();
        let store = IdentityDeletionStore::new(pool.clone());
        store.bind_legacy_issuer(ISSUER).await.unwrap();
        let command = request(central);
        assert_eq!(
            store
                .freeze(&command, &window())
                .await
                .unwrap()
                .reference_count,
            0
        );
        let orphan = Uuid::new_v4();
        let error = sqlx::query(
            "INSERT INTO wiki_identity_aliases(local_user_id,issuer,subject) VALUES($1,$2,$3)",
        )
        .bind(orphan)
        .bind(ISSUER)
        .bind(central.to_string())
        .execute(&pool)
        .await
        .unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().constraint(),
            Some("wiki_identity_deletion_fenced")
        );
        store.release(&command, &window()).await.unwrap();
        sqlx::query(
            "INSERT INTO wiki_identity_aliases(local_user_id,issuer,subject) VALUES($1,$2,$3)",
        )
        .bind(orphan)
        .bind(ISSUER)
        .bind(central.to_string())
        .execute(&pool)
        .await
        .unwrap();
        assert_eq!(
            store
                .check(&request(central), &window())
                .await
                .unwrap()
                .reference_count,
            1,
            "a retained proven central alias is itself a durable reference"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL"]
async fn malformed_typed_issuers_are_unknown_and_cannot_evade_a_frozen_subject() {
    isolated(false,|pool|async move{
  let store=IdentityDeletionStore::new(pool.clone());store.bind_legacy_issuer(ISSUER).await.unwrap();let subject=Uuid::new_v4();store.freeze(&request(subject),&window()).await.unwrap();
  for key in ["issuer","actor_issuer","owner_issuer","responsible_issuer"]{for bad in [serde_json::json!(123),serde_json::Value::Null,serde_json::json!(" "),serde_json::json!({"value":ISSUER}),serde_json::json!(false)]{
   let mut document=serde_json::json!({"actor_subject":subject});document[key]=bad;
   let unknown:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM wiki_identity_json_refs($1,$2) WHERE NOT resolved)").bind(document.clone()).bind(ISSUER).fetch_one(&pool).await.unwrap();assert!(unknown,"malformed {key} cannot fall back to the deployment issuer");
   let error=sqlx::query("INSERT INTO wiki_project_space_projection_sets(tracker_instance_id,project_id,revision,operation_id,command) VALUES($1,$2,1,$3,$4)").bind(Uuid::new_v4()).bind(Uuid::new_v4()).bind(Uuid::new_v4()).bind(document).execute(&pool).await.unwrap_err();assert_eq!(error.as_database_error().unwrap().constraint(),Some("wiki_identity_reference_unknown"));
  }}
  for bad in [serde_json::Value::Null,serde_json::json!(123),serde_json::json!({}),serde_json::json!([]),serde_json::json!(false),serde_json::json!(" ")] {
   let document=serde_json::json!({"actor_identity":{"issuer":ISSUER,"subject":bad}});
   let unknown:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM wiki_identity_json_refs($1,$2) WHERE NOT resolved)").bind(document.clone()).bind(ISSUER).fetch_one(&pool).await.unwrap();assert!(unknown,"malformed required subject cannot disappear from the inventory");
   let error=sqlx::query("INSERT INTO wiki_project_space_projection_sets(tracker_instance_id,project_id,revision,operation_id,command) VALUES($1,$2,1,$3,$4)").bind(Uuid::new_v4()).bind(Uuid::new_v4()).bind(Uuid::new_v4()).bind(document).execute(&pool).await.unwrap_err();assert_eq!(error.as_database_error().unwrap().constraint(),Some("wiki_identity_reference_unknown"));
  }
 }).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable PostgreSQL"]
async fn owner_freeze_and_release_cannot_commit_after_the_original_machine_window() {
    isolated(false, |pool| async move {
        let store = IdentityDeletionStore::new(pool.clone());
        store.bind_legacy_issuer(ISSUER).await.unwrap();
        verify_commit_deadlines(&pool, &store).await;
    })
    .await;
}

async fn verify_commit_deadlines(pool: &PgPool, store: &IdentityDeletionStore) {
    // The barrier is reached only by a deferred trigger during COMMIT, after
    // every Rust pre-commit check and the durable journal write have run.
    for releasing in [false, true] {
        let command = request(Uuid::new_v4());
        let original = if releasing {
            Some(store.freeze(&command, &window()).await.unwrap())
        } else {
            None
        };
        sqlx::raw_sql("CREATE SEQUENCE qa_owner_commit_seen;
            CREATE SEQUENCE qa_owner_commit_until; CREATE SEQUENCE qa_owner_commit_pid; CREATE SEQUENCE qa_owner_commit_valid;
            CREATE FUNCTION qa_owner_commit_delay() RETURNS trigger LANGUAGE plpgsql AS $$
            BEGIN PERFORM nextval('qa_owner_commit_seen');
            PERFORM setval('qa_owner_commit_until',floor(extract(epoch FROM NEW.valid_until)*1000000)::bigint,true);
            PERFORM setval('qa_owner_commit_pid',pg_backend_pid()::bigint,true);
            PERFORM setval('qa_owner_commit_valid',CASE WHEN clock_timestamp()<NEW.valid_until THEN 1 ELSE 2 END,true);
            PERFORM pg_advisory_xact_lock(714402216103::bigint); RETURN NEW; END $$;
            CREATE CONSTRAINT TRIGGER a_qa_owner_commit_delay AFTER INSERT ON wiki_request_commit_fences
            DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION qa_owner_commit_delay();")
            .execute(pool).await.unwrap();
        let mut barrier = pool.begin().await.unwrap();
        let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *barrier)
            .await
            .unwrap();
        sqlx::query("SELECT pg_advisory_xact_lock(714402216103::bigint)")
            .execute(&mut *barrier)
            .await
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let fixed = shared::identity_deletion::IdentityDeletionWindow::restricted(
            deadline,
            chrono::Utc::now().timestamp() + 30,
        );
        let snapshot_sql = "SELECT jsonb_build_object('journal',(SELECT COALESCE(jsonb_agg(to_jsonb(o)),'[]'::jsonb) FROM wiki_identity_deletion_operations o WHERE operation_id=$1),'epoch',(SELECT to_jsonb(e) FROM wiki_identity_fence_epoch e WHERE singleton))";
        let before: serde_json::Value = sqlx::query_scalar(snapshot_sql)
            .bind(command.operation_id)
            .fetch_one(pool)
            .await
            .unwrap();
        let task = {
            let store = store.clone();
            let command = command.clone();
            tokio::spawn(async move {
                if releasing {
                    store.release(&command, &fixed).await.map(|_| ())
                } else {
                    store.freeze(&command, &fixed).await.map(|_| ())
                }
            })
        };
        let _abort = OwnerCommitAbort(task.abort_handle());
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                let waiting: bool = sqlx::query_scalar("SELECT is_called AND $1=ANY(pg_blocking_pids(last_value::int)) FROM qa_owner_commit_pid")
                    .bind(pid).fetch_one(pool).await.unwrap();
                if waiting { break; }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }).await.expect("the exact owner backend must reach the real deferred COMMIT barrier");
        let entered: bool = sqlx::query_scalar("SELECT is_called FROM qa_owner_commit_seen")
            .fetch_one(pool)
            .await
            .unwrap();
        assert!(
            entered,
            "the test must not settle for a pre-commit rejection"
        );
        let valid_at_entry: i64 =
            sqlx::query_scalar("SELECT last_value FROM qa_owner_commit_valid")
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!(
            valid_at_entry, 1,
            "the actual DB commit cap must still be valid on deferred trigger entry"
        );
        let still_valid:bool=sqlx::query_scalar("SELECT clock_timestamp()<to_timestamp(last_value::double precision/1000000) FROM qa_owner_commit_until").fetch_one(pool).await.unwrap();
        assert!(
            still_valid,
            "the original DB deadline must be unexpired at the actual blocked COMMIT barrier"
        );
        tokio::time::timeout(std::time::Duration::from_secs(5),async {
            loop {
                let expired:bool=sqlx::query_scalar("SELECT clock_timestamp()>=to_timestamp(last_value::double precision/1000000) FROM qa_owner_commit_until").fetch_one(pool).await.unwrap();
                if expired { break; }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }).await.expect("the original fixed DB cap must expire without renewal");
        barrier.commit().await.unwrap();
        assert!(
            matches!(task.await.unwrap(), Err(OwnerError::Expired)),
            "the original monotonic deadline must abort COMMIT"
        );
        let after: serde_json::Value = sqlx::query_scalar(snapshot_sql)
            .bind(command.operation_id)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(
            after, before,
            "all operation fields and the epoch must roll back exactly"
        );
        let state: Option<String> = sqlx::query_scalar(
            "SELECT state FROM wiki_identity_deletion_operations WHERE operation_id=$1",
        )
        .bind(command.operation_id)
        .fetch_optional(pool)
        .await
        .unwrap();
        assert_eq!(
            state.as_deref(),
            if releasing { Some("frozen") } else { None },
            "expired COMMIT must roll back its journal/fence transition"
        );
        let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM wiki_request_commit_fences")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(rows, 0, "transient clock rows never accumulate");
        sqlx::raw_sql("DROP TRIGGER a_qa_owner_commit_delay ON wiki_request_commit_fences; DROP FUNCTION qa_owner_commit_delay(); DROP SEQUENCE qa_owner_commit_seen; DROP SEQUENCE qa_owner_commit_until; DROP SEQUENCE qa_owner_commit_pid; DROP SEQUENCE qa_owner_commit_valid;")
            .execute(pool).await.unwrap();
        let fresh = store.freeze(&command, &window()).await.unwrap();
        if let Some(original) = original {
            assert_eq!(
                fresh, original,
                "expired release must retain the exact original freeze"
            );
        }
        store.release(&command, &window()).await.unwrap();
    }
}

struct OwnerCommitAbort(tokio::task::AbortHandle);
impl Drop for OwnerCommitAbort {
    fn drop(&mut self) {
        self.0.abort();
    }
}
