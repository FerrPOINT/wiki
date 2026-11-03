//! Manual PostgreSQL wire-fault regression through the actual attachment backend port.
use shared::wiki_contract::WikiClaims;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::{Arc, atomic::Ordering};
use uuid::Uuid;

#[path = "support/commit_loss.rs"]
mod commit_loss;
use commit_loss::FaultProxy;

#[tokio::test]
#[ignore = "manual owned wiki_space_test PostgreSQL wire-fault regression"]
async fn lost_commit_reply_never_deletes_a_committed_attachment() {
    let url = std::env::var("WIKI_TEST_DATABASE_URL").expect("owned PostgreSQL fixture URL");
    let root = PgPool::connect(&url).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT current_database()")
            .fetch_one(&root)
            .await
            .unwrap(),
        "wiki_space_test"
    );
    let socket =
        std::env::var("WIKI_TEST_POSTGRES_SOCKET").expect("owned socket") + "/.s.PGSQL.5432";
    for refuse_readback in [false, true] {
        let schema = format!("wiki_attachment_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&root)
            .await
            .unwrap();
        let proxy = FaultProxy::start(socket.clone(), refuse_readback, false).await;
        let mut scoped = reqwest::Url::parse(&format!(
            "postgres://qa@127.0.0.1:{}/wiki_space_test?sslmode=disable",
            proxy.port
        ))
        .unwrap();
        scoped
            .query_pairs_mut()
            .append_pair("options", &format!("-csearch_path={schema}"));
        let mut config = shared::AppConfig::default();
        config.database.url = scoped.to_string();
        config.database.max_connections = 1;
        config.database.min_connections = 0;
        config.database.connect_timeout_seconds = 1;
        config.bootstrap.admin_email = None;
        config.bootstrap.admin_password = None;
        let storage_root =
            std::env::temp_dir().join(format!("wiki-upload-{}", Uuid::new_v4().simple()));
        let storage = Arc::new(infra::LocalWikiAttachmentStorage::new(&storage_root));
        let (backend, _) = infra::connect_postgres_wiki_backend(&config, storage)
            .await
            .unwrap();
        let mut direct = reqwest::Url::parse(&url).unwrap();
        direct
            .query_pairs_mut()
            .append_pair("options", &format!("-csearch_path={schema}"));
        let control = PgPoolOptions::new()
            .max_connections(3)
            .connect(direct.as_str())
            .await
            .unwrap();
        let user = Uuid::new_v4();
        sqlx::query("INSERT INTO users(id,email,username,display_name,password_hash,global_role) VALUES($1,$2,$3,'Fixture','!','admin')")
            .bind(user).bind(format!("{user}@example.test")).bind(user.to_string()).execute(&control).await.unwrap();
        let claims = WikiClaims {
            user_id: user.to_string(),
            session_id: None,
            request_id: None,
            admission: None,
        };
        // Registration/migrations are complete. Lose only the actual upload COMMIT reply.
        proxy.armed.store(true, Ordering::SeqCst);
        let bytes = b"preserve committed bytes".to_vec();
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            backend.upload_attachment(
                &claims,
                "Кавычки ' файл.txt".into(),
                "text/plain".into(),
                bytes.clone(),
            ),
        )
        .await
        .unwrap();
        assert!(
            proxy.lost.load(Ordering::SeqCst),
            "the upload COMMIT reply must actually be lost"
        );
        let (id, key): (Uuid, String) =
            sqlx::query_as("SELECT id,storage_key FROM attachments WHERE uploaded_by=$1")
                .bind(user)
                .fetch_one(&control)
                .await
                .unwrap();
        assert_eq!(
            tokio::fs::read(storage_root.join(&key)).await.unwrap(),
            bytes,
            "unknown COMMIT must never remove the committed file"
        );
        if refuse_readback {
            assert!(
                matches!(result, Err(shared::AppError::Unavailable(_))),
                "unavailable readback must preserve uncertainty: {result:?}"
            );
        } else {
            assert_eq!(
                result.unwrap().id,
                id.to_string(),
                "readback returns the original ID"
            );
            // A genuine pre-COMMIT refusal is safe to clean up, leaving the first upload intact.
            sqlx::raw_sql("CREATE FUNCTION reject_upload() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'fixture refusal'; END $$; CREATE TRIGGER reject_upload BEFORE INSERT ON attachments FOR EACH ROW EXECUTE FUNCTION reject_upload()")
                .execute(&control).await.unwrap();
            assert!(
                backend
                    .upload_attachment(
                        &claims,
                        "rejected.txt".into(),
                        "text/plain".into(),
                        b"rejected".to_vec()
                    )
                    .await
                    .is_err()
            );
            assert_eq!(
                std::fs::read_dir(&storage_root).unwrap().count(),
                1,
                "only confirmed rollback bytes are removed"
            );
        }
        drop(backend);
        control.close().await;
        drop(proxy);
        std::fs::remove_dir_all(&storage_root).unwrap();
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&root)
            .await
            .unwrap();
    }
    root.close().await;
}
