//! Native request-to-commit race checks, using only the owned synthetic schema.
use super::*;
use pdlc_shared::space_access::*;
use shared::wiki_space_access::{WikiSpaceAdmission, WikiSpaceMapping, WikiSpaceSubject};

async fn wait_for_writer_locks(pool: &PgPool, expected: i64) {
    for _ in 0..80 {
        let waiting: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_stat_activity WHERE datname=current_database() AND pid<>pg_backend_pid() AND wait_event_type='Lock'",
        ).fetch_one(pool).await.unwrap();
        if waiting >= expected {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!("writer did not reach the held database lock");
}

pub(super) async fn verify(
    backend: &PostgresWikiBackend,
    user: Uuid,
    central: Uuid,
    mappings: &[(Uuid, SpaceRef)],
    document: Uuid,
) {
    let pool = &backend.pool;
    let local = mappings[0].0;
    sqlx::query("INSERT INTO space_members(space_id,user_id,role) VALUES($1,$2,'editor')")
        .bind(local)
        .bind(user)
        .execute(pool)
        .await
        .unwrap();
    let actor = super::admission_tests::claims(
        user,
        central,
        "user",
        mappings,
        &[mappings[0].1.clone()],
        &[local],
    );
    let mut barrier = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM spaces WHERE id=$1 FOR UPDATE")
        .bind(local)
        .execute(&mut *barrier)
        .await
        .unwrap();
    let writer = backend.clone();
    let old = actor.clone();
    let task = tokio::spawn(async move {
        let body = serde_json::from_value(
            serde_json::json!({"title":"Role wait must roll back","slug":"role-wait-no-commit"}),
        )
        .unwrap();
        writer.create_document(&old, "QA01", body).await
    });
    wait_for_writer_locks(pool, 1).await;
    assert!(!task.is_finished());
    sqlx::query("DELETE FROM space_members WHERE space_id=$1 AND user_id=$2")
        .bind(local)
        .bind(user)
        .execute(pool)
        .await
        .unwrap();
    barrier.commit().await.unwrap();
    assert!(
        matches!(task.await.unwrap(), Err(shared::AppError::Forbidden)),
        "role revoked while waiting must prevent the actual document write"
    );
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM documents WHERE slug='role-wait-no-commit'")
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
    sqlx::query("INSERT INTO space_members(space_id,user_id,role) VALUES($1,$2,'editor')")
        .bind(local)
        .bind(user)
        .execute(pool)
        .await
        .unwrap();
    let now = chrono::Utc::now().timestamp();
    let identity = CentralIdentity {
        issuer: "https://auth.example.test".into(),
        subject: central.to_string(),
    };
    let short = WikiClaims {
        user_id: user.to_string(),
        session_id: Some(Uuid::new_v4().to_string()),
        request_id: None,
        admission: Some(Arc::new(
            WikiSpaceAdmission::confirmed(
                WikiSpaceSubject {
                    user_id: user,
                    identity: identity.clone(),
                    token_expiry: now + 2,
                    global_role: "user".into(),
                    may_read: true,
                    may_write: true,
                },
                SpaceAdmission {
                    schema_version: 1,
                    identity,
                    identity_active: true,
                    policy_revision: 1,
                    policy: SpaceAccessPolicy::Selected {
                        spaces: vec![mappings[0].1.clone()],
                    },
                    validity: AdmissionValidity::confirmed(now, now + 2, i64::MAX).unwrap(),
                    spaces: vec![ConfirmedSpace {
                        space: mappings[0].1.clone(),
                        revision: 10,
                        state: SpaceState::Active,
                    }],
                },
                vec![WikiSpaceMapping {
                    local_space_id: local,
                    space: mappings[0].1.clone(),
                    may_edit: true,
                    may_admin: false,
                }],
            )
            .unwrap(),
        )),
    };
    let before: (String,String) = sqlx::query_as("SELECT d.title,dd.content_markdown FROM documents d JOIN document_drafts dd ON dd.document_id=d.id WHERE d.id=$1")
        .bind(document).fetch_one(pool).await.unwrap();
    let mut barrier = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM documents WHERE id=$1 FOR UPDATE")
        .bind(document)
        .execute(&mut *barrier)
        .await
        .unwrap();
    let writer = backend.clone();
    let id = document.to_string();
    let task = tokio::spawn(async move {
        let body = serde_json::from_value(
            serde_json::json!({"title":"Expired write","content_markdown":"Must roll back"}),
        )
        .unwrap();
        writer.update_document_draft(&short, &id, body).await
    });
    wait_for_writer_locks(pool, 1).await;
    while chrono::Utc::now().timestamp() <= now + 2 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(!task.is_finished());
    barrier.commit().await.unwrap();
    assert!(
        matches!(task.await.unwrap(), Err(shared::AppError::Forbidden)),
        "expired original admission must roll back changes after a document row wait"
    );
    let after: (String,String) = sqlx::query_as("SELECT d.title,dd.content_markdown FROM documents d JOIN document_drafts dd ON dd.document_id=d.id WHERE d.id=$1")
        .bind(document).fetch_one(pool).await.unwrap();
    assert_eq!(after, before);
    let fresh = super::admission_tests::claims(
        user,
        central,
        "user",
        mappings,
        &[mappings[0].1.clone()],
        &[local],
    );
    let body =
        serde_json::from_value(serde_json::json!({"title":"Current writer succeeds"})).unwrap();
    let created = backend.create_document(&fresh, "QA01", body).await.unwrap();
    assert!(created.can_edit);

    // Independent documents share the Space lifecycle lock without a late upgrade.
    let other: Uuid = created.id.parse().unwrap();
    let mut barrier = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM documents WHERE id=ANY($1) ORDER BY id FOR UPDATE")
        .bind(vec![document, other])
        .execute(&mut *barrier)
        .await
        .unwrap();
    let spawn = |id: Uuid, title: &str| {
        let backend = backend.clone();
        let actor = fresh.clone();
        let title = title.to_string();
        tokio::spawn(async move {
            let body = serde_json::from_value(
                serde_json::json!({"title":title,"content_markdown":"Concurrent independent edit"}),
            )
            .unwrap();
            backend
                .update_document_draft(&actor, &id.to_string(), body)
                .await
        })
    };
    let first = spawn(document, "Independent first writer");
    let second = spawn(other, "Independent second writer");
    wait_for_writer_locks(pool, 2).await;
    barrier.commit().await.unwrap();
    let both = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        (first.await.unwrap(), second.await.unwrap())
    })
    .await
    .expect("independent document writers must not deadlock");
    assert!(both.0.is_ok(), "{:?}", both.0);
    assert!(both.1.is_ok(), "{:?}", both.1);

    sqlx::query("DELETE FROM space_members WHERE space_id=$1 AND user_id=$2")
        .bind(local)
        .bind(user)
        .execute(pool)
        .await
        .unwrap();
}
