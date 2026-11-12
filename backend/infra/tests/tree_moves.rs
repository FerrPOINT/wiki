//! Manual native test of the public backend port, with independent synthetic storage.
use pdlc_shared::space_access::*;
use shared::{
    wiki_contract::{MoveDocumentRequest, WikiClaims},
    wiki_space_access::{WikiSpaceAdmission, WikiSpaceMapping, WikiSpaceSubject},
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use uuid::Uuid;

fn actor(user: Uuid, central: Uuid, spaces: &[(Uuid, SpaceRef)]) -> WikiClaims {
    let now = chrono::Utc::now().timestamp();
    let identity = CentralIdentity {
        issuer: "https://auth.example.test".into(),
        subject: central.to_string(),
    };
    let admission = SpaceAdmission {
        inheritance: None,
        schema_version: 1,
        identity: identity.clone(),
        identity_active: true,
        policy_revision: 1,
        policy: SpaceAccessPolicy::Selected {
            spaces: spaces.iter().map(|(_, s)| s.clone()).collect(),
        },
        validity: AdmissionValidity::confirmed(now, now + 60, i64::MAX).unwrap(),
        spaces: spaces
            .iter()
            .map(|(_, space)| ConfirmedSpace {
                space: space.clone(),
                revision: 1,
                state: SpaceState::Active,
            })
            .collect(),
    };
    WikiClaims {
        user_id: user.to_string(),
        session_id: Some(Uuid::new_v4().to_string()),
        request_id: None,
        admission: Some(Arc::new(
            WikiSpaceAdmission::confirmed(
                WikiSpaceSubject {
                    user_id: user,
                    identity,
                    token_expiry: now + 60,
                    global_role: "user".into(),
                    may_read: true,
                    may_write: true,
                },
                admission,
                spaces
                    .iter()
                    .map(|(local, space)| WikiSpaceMapping {
                        local_space_id: *local,
                        space: space.clone(),
                        may_edit: true,
                        may_admin: false,
                    })
                    .collect(),
            )
            .unwrap(),
        )),
    }
}
async fn waiting(pool: &PgPool, count: i64) {
    tokio::time::timeout(std::time::Duration::from_secs(2),async {
        loop {let blocked:i64=sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE datname=current_database() AND wait_event_type='Lock' AND pid<>pg_backend_pid()")
            .fetch_one(pool).await.unwrap();if blocked>=count {break;}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
    }).await.expect("writers must wait for their space tree lock");
}
#[tokio::test]
#[ignore = "manual owned wiki_space_test PostgreSQL regression"]
async fn opposite_moves_cannot_cycle_and_editors_keep_independent_space_access() {
    let url = std::env::var("WIKI_TEST_DATABASE_URL").expect("owned PostgreSQL fixture URL");
    let root = PgPool::connect(&url).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT current_database()")
            .fetch_one(&root)
            .await
            .unwrap(),
        "wiki_space_test"
    );
    let schema = format!("wiki_tree_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&root)
        .await
        .unwrap();
    let mut scoped = reqwest::Url::parse(&url).unwrap();
    scoped
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    let mut config = shared::AppConfig::default();
    config.database.url = scoped.to_string();
    config.database.max_connections = 5;
    config.database.min_connections = 0;
    config.bootstrap.admin_email = None;
    config.bootstrap.admin_password = None;
    let storage = Arc::new(infra::LocalWikiAttachmentStorage::new(
        std::env::temp_dir().join(format!("wiki-tree-{}", Uuid::new_v4().simple())),
    ));
    let (backend, _) = infra::connect_postgres_wiki_backend(&config, storage)
        .await
        .unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database.url)
        .await
        .unwrap();
    let editor = Uuid::new_v4();
    let central = Uuid::new_v4();
    let owner = Uuid::new_v4();
    for (user, role, sub) in [(editor, "user", central), (owner, "admin", Uuid::new_v4())] {
        sqlx::query("INSERT INTO users(id,email,username,display_name,password_hash,global_role,central_sub) VALUES($1,$2,$3,'Fixture','!',$4,$5)")
            .bind(user).bind(format!("{user}@example.test")).bind(user.to_string()).bind(role).bind(sub.to_string()).execute(&pool).await.unwrap();
    }
    let registry: Uuid = std::env::var("WIKI_NAMESPACE__REGISTRY_INSTANCE_ID")
        .expect("owned fixture registry")
        .parse()
        .unwrap();
    let mut spaces = Vec::new();
    let mut docs = Vec::new();
    for key in ["QA01", "QA02"] {
        let local = Uuid::new_v4();
        let public = SpaceRef {
            registry_instance_id: registry,
            space_id: Uuid::new_v4(),
        };
        sqlx::query("INSERT INTO spaces(id,key,name,owner_id) VALUES($1,$2,'Tree fixture',$3)")
            .bind(local)
            .bind(key)
            .bind(owner)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO wiki_space_catalog(registry_instance_id,space_id,local_space_id,admin_revision,state,operation_id) VALUES($1,$2,$3,1,'active',$4)").bind(registry).bind(public.space_id).bind(local).bind(Uuid::new_v4()).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO space_members(space_id,user_id,role) VALUES($1,$2,'editor')")
            .bind(local)
            .bind(editor)
            .execute(&pool)
            .await
            .unwrap();
        for title in ["A", "B"] {
            let doc = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO documents(id,space_id,slug,title,owner_id) VALUES($1,$2,$3,$3,$4)",
            )
            .bind(doc)
            .bind(local)
            .bind(title.to_lowercase())
            .bind(editor)
            .execute(&pool)
            .await
            .unwrap();
            docs.push(doc);
        }
        spaces.push((local, public));
    }
    let claims = actor(editor, central, &spaces);
    let (a, b) = (docs[0], docs[1]);
    let mut barrier = pool.begin().await.unwrap();
    // Delay B's publication without blocking the FK key-share acquired by
    // A->B. Both callers have passed the old, external ancestor preflight.
    sqlx::query("SELECT id FROM documents WHERE id=$1 FOR NO KEY UPDATE")
        .bind(b)
        .execute(&mut *barrier)
        .await
        .unwrap();
    let right = backend.clone();
    let c = claims.clone();
    let second = tokio::spawn(async move {
        right
            .move_document(
                &c,
                &b.to_string(),
                MoveDocumentRequest {
                    parent_id: Some(a.to_string()),
                },
            )
            .await
    });
    waiting(&pool, 1).await;
    let left = backend.clone();
    let c = claims.clone();
    let first = tokio::spawn(async move {
        left.move_document(
            &c,
            &a.to_string(),
            MoveDocumentRequest {
                parent_id: Some(b.to_string()),
            },
        )
        .await
    });
    tokio::time::timeout(std::time::Duration::from_secs(2),async {
        loop {
            // Baseline A publishes while B waits; fixed A waits for B's space
            // lock. Either observable point makes releasing B deterministic.
            if first.is_finished() {break;}
            let blocked:i64=sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE datname=current_database() AND wait_event_type='Lock' AND pid<>pg_backend_pid()")
                .fetch_one(&pool).await.unwrap();
            if blocked>=2 {break;}
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }).await.unwrap();
    barrier.commit().await.unwrap();
    let (first, second) = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        (first.await.unwrap(), second.await.unwrap())
    })
    .await
    .unwrap();
    let parents: Vec<(Uuid, Option<Uuid>)> =
        sqlx::query_as("SELECT id,parent_id FROM documents WHERE space_id=$1 ORDER BY id")
            .bind(spaces[0].0)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert!(
        !parents
            .iter()
            .all(|(id, parent)| *parent == Some(if *id == a { b } else { a })),
        "cycle survived serialized validation"
    );
    assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
    assert!(
        matches!(first, Err(shared::AppError::InvalidInput(_)))
            || matches!(second, Err(shared::AppError::InvalidInput(_))),
        "unexpected move outcomes: {first:?}; {second:?}"
    );
    sqlx::query("UPDATE documents SET parent_id=NULL WHERE space_id=$1")
        .bind(spaces[0].0)
        .execute(&pool)
        .await
        .unwrap();
    let mut barrier = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM spaces WHERE id=$1 FOR SHARE")
        .bind(spaces[0].0)
        .execute(&mut *barrier)
        .await
        .unwrap();
    let writer = backend.clone();
    let c = claims.clone();
    let moving = tokio::spawn(async move {
        writer
            .move_document(
                &c,
                &a.to_string(),
                MoveDocumentRequest {
                    parent_id: Some(b.to_string()),
                },
            )
            .await
    });
    waiting(&pool, 1).await;
    tokio::time::timeout(
        std::time::Duration::from_secs(1),
        backend.move_document(
            &claims,
            &docs[2].to_string(),
            MoveDocumentRequest {
                parent_id: Some(docs[3].to_string()),
            },
        ),
    )
    .await
    .unwrap()
    .unwrap();
    sqlx::query("DELETE FROM space_members WHERE space_id=$1 AND user_id=$2")
        .bind(spaces[0].0)
        .bind(editor)
        .execute(&pool)
        .await
        .unwrap();
    barrier.commit().await.unwrap();
    assert!(matches!(
        moving.await.unwrap(),
        Err(shared::AppError::Forbidden)
    ));
    sqlx::query("INSERT INTO space_members(space_id,user_id,role) VALUES($1,$2,'editor')")
        .bind(spaces[0].0)
        .bind(editor)
        .execute(&pool)
        .await
        .unwrap();
    backend
        .move_document(
            &claims,
            &a.to_string(),
            MoveDocumentRequest {
                parent_id: Some(b.to_string()),
            },
        )
        .await
        .unwrap();
    let tree = backend.get_space_tree(&claims, "QA01").await.unwrap();
    assert_eq!(tree.documents.len(), 1);
    assert_eq!(tree.documents[0].id, b.to_string());
    assert_eq!(tree.documents[0].children.len(), 1);
    assert_eq!(tree.documents[0].children[0].id, a.to_string());
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM documents")
            .fetch_one(&pool)
            .await
            .unwrap(),
        4
    );
    drop(backend);
    pool.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&root)
        .await
        .unwrap();
    root.close().await;
}
