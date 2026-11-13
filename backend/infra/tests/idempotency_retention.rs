//! Manual owner maintenance and per-key expiry regression; no production storage.
use shared::wiki_contract::{WikiIdempotencyRequest, WikiIdempotencyStatus};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use uuid::Uuid;
#[tokio::test]
#[ignore = "manual: actual owned PostgreSQL"]
async fn expired_commands_are_cleaned_in_batches_without_reexecuting_unknown_outcomes() {
    let raw = std::env::var("WIKI_TEST_DATABASE_URL").unwrap();
    let root = PgPool::connect(&raw).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT current_database()")
            .fetch_one(&root)
            .await
            .unwrap(),
        "wiki_space_test"
    );
    let schema = format!("wiki_idempotency_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&root)
        .await
        .unwrap();
    let mut url = reqwest::Url::parse(&raw).unwrap();
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    let mut config = shared::AppConfig::default();
    config.database.url = url.to_string();
    config.database.max_connections = 2;
    config.database.min_connections = 0;
    config.bootstrap.admin_email = None;
    config.bootstrap.admin_password = None;
    config.maintenance.batch_size = 25;
    let path = std::env::temp_dir().join(format!("wiki-retention-{}", Uuid::new_v4().simple()));
    let (backend, _) = infra::connect_postgres_wiki_backend(
        &config,
        Arc::new(infra::LocalWikiAttachmentStorage::new(&path)),
    )
    .await
    .unwrap();
    let control = PgPoolOptions::new()
        .max_connections(3)
        .connect(url.as_str())
        .await
        .unwrap();
    let result=tokio::spawn(async move{
        let user=Uuid::new_v4();
        sqlx::query("INSERT INTO users(id,email,username,display_name,password_hash,global_role) VALUES($1,'retention@example.test','retention','Retention','!','admin')")
            .bind(user).execute(&control).await.unwrap();
        sqlx::query("INSERT INTO idempotency_records(id,actor_id,idempotency_key,method,path,request_hash,state,response_status,response_body,expires_at) SELECT gen_random_uuid(),$1,'expired-'||n,'POST','/fixture','fingerprint','completed',200,'{}',now()-interval '1 day' FROM generate_series(1,253)n")
            .bind(user).execute(&control).await.unwrap();
        sqlx::query("INSERT INTO idempotency_records(id,actor_id,idempotency_key,method,path,request_hash,state,expires_at) VALUES(gen_random_uuid(),$1,'unknown','POST','/fixture','fingerprint','processing',now()-interval '2 days')")
            .bind(user).execute(&control).await.unwrap();
        let request=|key:&str|WikiIdempotencyRequest{actor_id:user.to_string(),key:key.into(),method:"POST".into(),path:"/fixture".into(),request_hash:"fingerprint".into()};
        assert!(matches!(backend.begin_idempotent_request(request("new")).await.unwrap(),WikiIdempotencyStatus::Started));
        let count:i64=sqlx::query_scalar("SELECT count(*) FROM idempotency_records WHERE state='completed'").fetch_one(&control).await.unwrap();
        assert_eq!(count,253,"a normal request must not sweep all expired records");
        assert!(matches!(backend.begin_idempotent_request(request("unknown")).await,Err(shared::AppError::Conflict(_))));
        let report=backend.run_maintenance().await.unwrap();
        assert_eq!(report.expired_idempotency_records_deleted,25);
        let state:String=sqlx::query_scalar("SELECT state FROM idempotency_records WHERE idempotency_key='unknown'").fetch_one(&control).await.unwrap();
        assert_eq!(state,"processing","unknown legacy outcomes require inventory/readback");
        sqlx::query("INSERT INTO idempotency_records(id,actor_id,idempotency_key,method,path,request_hash,state,response_status,response_body,expires_at) VALUES(gen_random_uuid(),$1,'renew','POST','/fixture','old-fingerprint','completed',200,'{}',now()-interval '1 day')")
            .bind(user).execute(&control).await.unwrap();
        assert!(matches!(backend.begin_idempotent_request(request("renew")).await.unwrap(),WikiIdempotencyStatus::Started),
            "expiry permits reuse of this confirmed completed key only");
        let mut held=control.begin().await.unwrap();
        let retained:Uuid=sqlx::query_scalar("SELECT id FROM idempotency_records WHERE state='completed' ORDER BY expires_at,id LIMIT 1 FOR UPDATE").fetch_one(&mut *held).await.unwrap();
        assert_eq!(backend.run_maintenance().await.unwrap().expired_idempotency_records_deleted,25);
        assert!(sqlx::query_scalar::<_,bool>("SELECT EXISTS(SELECT 1 FROM idempotency_records WHERE id=$1)").bind(retained).fetch_one(&control).await.unwrap());
        held.rollback().await.unwrap();
        let mut deleted=50;
        for _ in 0..10 {let report=backend.run_maintenance().await.unwrap();assert!(report.expired_idempotency_records_deleted<=25);deleted+=report.expired_idempotency_records_deleted;}
        assert_eq!(deleted,253);
        assert_eq!(backend.run_maintenance().await.unwrap().expired_idempotency_records_deleted,0);
        control.close().await;
    }).await;
    if path.exists() {
        std::fs::remove_dir_all(path).unwrap();
    }
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&root)
        .await
        .unwrap();
    root.close().await;
    result.unwrap();
}
