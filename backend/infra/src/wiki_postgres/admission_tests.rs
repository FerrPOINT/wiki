//! Native read surfaces on populated synthetic data; this is not a deployment receipt.
use super::*;
use domain::wiki::WikiAttachmentStorage;
use pdlc_shared::space_access::*;
use shared::wiki_space_access::{WikiSpaceAdmission, WikiSpaceMapping, WikiSpaceSubject};
use sqlx::postgres::PgPoolOptions;

#[allow(clippy::too_many_arguments)]
fn claims_with_write_scope(
    user: Uuid,
    central: Uuid,
    role: &str,
    spaces: &[(Uuid, SpaceRef)],
    selected: &[SpaceRef],
    edit: &[Uuid],
    space_revision: i64,
    may_write: bool,
) -> WikiClaims {
    let now = chrono::Utc::now().timestamp();
    let identity = CentralIdentity {
        issuer: "https://auth.example.test".into(),
        subject: central.to_string(),
    };
    let admission = SpaceAdmission {
        schema_version: 1,
        identity: identity.clone(),
        identity_active: true,
        policy_revision: 1,
        policy: SpaceAccessPolicy::Selected {
            spaces: selected.to_vec(),
        },
        validity: AdmissionValidity::confirmed(now, now + 60, i64::MAX).unwrap(),
        spaces: spaces
            .iter()
            .map(|(_, space)| ConfirmedSpace {
                space: space.clone(),
                revision: space_revision,
                state: SpaceState::Active,
            })
            .collect(),
    };
    let subject = WikiSpaceSubject {
        user_id: user,
        identity,
        token_expiry: now + 60,
        global_role: role.into(),
        may_read: true,
        may_write,
    };
    WikiClaims {
        user_id: user.to_string(),
        session_id: Some(Uuid::new_v4().to_string()),
        request_id: None,
        admission: Some(Arc::new(
            WikiSpaceAdmission::confirmed(
                subject,
                admission,
                spaces
                    .iter()
                    .map(|(local, space)| WikiSpaceMapping {
                        local_space_id: *local,
                        space: space.clone(),
                        may_edit: role == "admin" || edit.contains(local),
                        may_admin: role == "admin",
                    })
                    .collect(),
            )
            .unwrap(),
        )),
    }
}

fn claims_at_revision(
    user: Uuid,
    central: Uuid,
    role: &str,
    spaces: &[(Uuid, SpaceRef)],
    selected: &[SpaceRef],
    edit: &[Uuid],
    space_revision: i64,
) -> WikiClaims {
    claims_with_write_scope(
        user,
        central,
        role,
        spaces,
        selected,
        edit,
        space_revision,
        true,
    )
}

pub(super) fn claims(
    user: Uuid,
    central: Uuid,
    role: &str,
    spaces: &[(Uuid, SpaceRef)],
    selected: &[SpaceRef],
    edit: &[Uuid],
) -> WikiClaims {
    claims_at_revision(user, central, role, spaces, selected, edit, 10)
}

async fn assert_dossier_document_visibility(
    backend: &PostgresWikiBackend,
    actor: &WikiClaims,
    include_drafts: bool,
) {
    let expected = if include_drafts { 2 } else { 1 };
    let tasks = backend.list_tasks(actor, "QA01").await.unwrap();
    assert_eq!(
        tasks
            .tasks
            .iter()
            .find(|x| x.task_key == "DOS-1")
            .unwrap()
            .document_count,
        expected
    );
    let phases = backend.list_phases(actor, "QA01").await.unwrap();
    assert_eq!(
        phases
            .phases
            .iter()
            .find(|x| x.phase_key == "phase-1")
            .unwrap()
            .document_count,
        expected
    );
    let task = backend.get_task(actor, "QA01", "DOS-1").await.unwrap();
    assert_eq!(
        task.documents.len(),
        expected,
        "draft metadata must not escape through task detail"
    );
    let phase = backend.get_phase(actor, "QA01", "phase-1").await.unwrap();
    assert_eq!(
        phase.documents.len(),
        expected,
        "phase detail uses the same document visibility"
    );
    if !include_drafts {
        assert_eq!(task.title.as_deref(), Some("Needle publication"));
        for document in task.documents.iter().chain(phase.documents.iter()) {
            assert_eq!(document.title, "Needle publication");
            assert_eq!(document.status, "published");
        }
    }
    let task_documents = backend
        .list_task_documents(actor, "QA01", "DOS-1")
        .await
        .unwrap();
    let phase_documents = backend
        .list_phase_documents(actor, "QA01", "phase-1")
        .await
        .unwrap();
    assert_eq!(task_documents.documents.len(), expected);
    assert_eq!(phase_documents.documents.len(), expected);
    for document in task_documents
        .documents
        .iter()
        .chain(phase_documents.documents.iter())
    {
        if !include_drafts {
            assert_eq!(document.title, "Needle publication");
            assert_eq!(document.status, "published");
            assert!(document.draft_markdown.is_empty());
            assert!(document.draft_checksum.is_none());
        }
    }
    for q in ["Draft-only", "unpublished-title"] {
        let query = || serde_json::from_value(serde_json::json!({"q":q,"limit":1})).unwrap();
        let tasks = backend
            .list_task_summaries(actor, "QA01", query())
            .await
            .unwrap();
        let phases = backend
            .list_phase_summaries(actor, "QA01", query())
            .await
            .unwrap();
        let total = if include_drafts {
            if q == "Draft-only" { 2 } else { 1 }
        } else {
            0
        };
        assert_eq!(
            tasks.total, total,
            "task search/count must filter private metadata before paging"
        );
        assert_eq!(
            phases.total, total,
            "phase search/count must filter private metadata before paging"
        );
        assert_eq!(tasks.tasks.len(), usize::from(total > 0));
        assert_eq!(phases.phases.len(), usize::from(total > 0));
    }
    let query = || serde_json::from_value(serde_json::json!({"q":"Needle","limit":1})).unwrap();
    assert_eq!(
        backend
            .list_task_summaries(actor, "QA01", query())
            .await
            .unwrap()
            .total,
        1
    );
    assert_eq!(
        backend
            .list_phase_summaries(actor, "QA01", query())
            .await
            .unwrap()
            .total,
        1
    );
    let query = || serde_json::from_value(serde_json::json!({"limit":100})).unwrap();
    let summaries = backend
        .list_task_summaries(actor, "QA01", query())
        .await
        .unwrap();
    let summary = summaries
        .tasks
        .iter()
        .find(|x| x.task_key == "DOS-1")
        .unwrap();
    assert_eq!(summary.document_count, expected);
    if !include_drafts {
        assert_eq!(summary.title.as_deref(), Some("Needle publication"));
    }
    let summaries = backend
        .list_phase_summaries(actor, "QA01", query())
        .await
        .unwrap();
    assert_eq!(
        summaries
            .phases
            .iter()
            .find(|x| x.phase_key == "phase-1")
            .unwrap()
            .document_count,
        expected
    );
}

#[tokio::test]
#[ignore = "requires owned wiki_space_test PostgreSQL database"]
async fn unified_admission_filters_before_paging_and_preserves_roles_revisions_and_downloads() {
    let url = std::env::var("WIKI_TEST_DATABASE_URL").expect("owned PostgreSQL URL");
    let root = PgPool::connect(&url).await.unwrap();
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&root)
        .await
        .unwrap();
    assert_eq!(database, "wiki_space_test", "refuse working database");
    let schema = format!("wiki_admission_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&root)
        .await
        .unwrap();
    let search = schema.clone();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .after_connect(move |connection, _| {
            let schema = search.clone();
            Box::pin(async move {
                sqlx::query(&format!("SET search_path TO {schema}"))
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect(&url)
        .await
        .unwrap();
    sqlx::migrate::Migrator::new(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../migrations"),
    )
    .await
    .unwrap()
    .run(&pool)
    .await
    .unwrap();
    let user = Uuid::new_v4();
    let admin = Uuid::new_v4();
    let central = Uuid::new_v4();
    let admin_central = Uuid::new_v4();
    for (id, role, subject) in [(user, "user", central), (admin, "admin", admin_central)] {
        sqlx::query("INSERT INTO users(id,email,username,display_name,password_hash,global_role,central_sub) VALUES($1,$2,$3,'Same name','!',$4,$5)")
            .bind(id).bind(format!("{id}@example.test")).bind(id.to_string()).bind(role).bind(subject.to_string()).execute(&pool).await.unwrap();
    }
    let storage_root = std::env::temp_dir().join(format!("wiki-admission-{}", Uuid::new_v4()));
    let storage = Arc::new(crate::wiki_storage::LocalWikiAttachmentStorage::new(
        &storage_root,
    ));
    let config = shared::AppConfig::default();
    let mut backend = PostgresWikiBackend {
        pool: pool.clone(),
        task_reader: None,
        auth: config.auth.clone(),
        storage: storage.clone(),
        max_upload_bytes: config.storage.max_upload_bytes,
        staged_attachment_ttl_hours: config.maintenance.staged_attachment_ttl_hours,
        maintenance_batch_size: config.maintenance.batch_size,
        settings: WikiSettingsSnapshot::from_config(&config),
    };
    let registry: Uuid = std::env::var("WIKI_NAMESPACE__REGISTRY_INSTANCE_ID")
        .expect("QA registry")
        .parse()
        .unwrap();
    let wiki_instance: Uuid = std::env::var("WIKI_NAMESPACE__INSTANCE_ID")
        .expect("QA Wiki instance")
        .parse()
        .unwrap();
    let task = shared::managed_links::TaskRef {
        tracker_instance_id: Uuid::new_v4(),
        task_id: Uuid::new_v4(),
    };

    let mut mappings = Vec::new();
    let mut documents = Vec::new();
    let mut revisions = Vec::new();
    let mut attachments = Vec::new();
    for key in ["QA01", "QA02"] {
        let space = Uuid::new_v4();
        let public = SpaceRef {
            registry_instance_id: registry,
            space_id: Uuid::new_v4(),
        };
        assert_ne!(space, public.space_id);
        sqlx::query("INSERT INTO spaces(id,key,name,owner_id) VALUES($1,$2,'Same name',$3)")
            .bind(space)
            .bind(key)
            .bind(admin)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO wiki_space_catalog(registry_instance_id,space_id,local_space_id,admin_revision,state,operation_id) VALUES($1,$2,$3,10,'active',$4)")
            .bind(registry).bind(public.space_id).bind(space).bind(Uuid::new_v4()).execute(&pool).await.unwrap();
        let document = Uuid::new_v4();
        let revision = Uuid::new_v4();
        let attachment = Uuid::new_v4();
        sqlx::query("INSERT INTO documents(id,space_id,slug,title,owner_id) VALUES($1,$2,'same-slug','Needle publication',$3)").bind(document).bind(space).bind(admin).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO document_revisions(id,document_id,version,title,content_markdown,content_html,content_text,content_checksum,author_id) VALUES($1,$2,1,'Needle publication',$3,$3,$3,'synthetic-checksum',$4)")
            .bind(revision).bind(document).bind(format!("Needle {key} body")).bind(admin).execute(&pool).await.unwrap();
        sqlx::query("UPDATE documents SET current_revision_id=$2,status='published' WHERE id=$1")
            .bind(document)
            .bind(revision)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO document_drafts(document_id,author_id,content_markdown) VALUES($1,$2,'private unpublished draft')").bind(document).bind(admin).execute(&pool).await.unwrap();
        storage.put(key, b"synthetic attachment").await.unwrap();
        sqlx::query("INSERT INTO attachments(id,space_id,owner_entity_type,owner_entity_id,file_name,content_type,size_bytes,storage_key,checksum,uploaded_by) VALUES($1,$2,'document',$3,'same.txt','text/plain',20,$4,'synthetic-checksum',$5)")
            .bind(attachment).bind(space).bind(document).bind(key).bind(admin).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO evidence_items(id,space_id,document_id,evidence_type,title,url,created_by) VALUES($1,$2,$3,'external_url','Needle evidence','https://example.test/evidence',$4)")
            .bind(Uuid::new_v4()).bind(space).bind(document).bind(admin).execute(&pool).await.unwrap();
        let command = pdlc_shared::resource_context::OwnerCommand {
            schema_version: 1,
            namespace: pdlc_shared::resource_context::NamespaceRef {
                registry_instance_id: registry,
                namespace_id: public.space_id,
            },
            resource: pdlc_shared::resource_context::ResourceRef {
                kind: pdlc_shared::resource_context::ResourceKind::WikiSpace,
                instance_id: wiki_instance,
                resource_id: space,
            },
            operation_id: Uuid::new_v4(),
            generation: 10,
            state: "active".into(),
            create_spec: None,
        };
        sqlx::query("INSERT INTO wiki_namespace_bindings(resource_id,registry_instance_id,namespace_id,generation,state,command) VALUES($1,$2,$3,10,'active',$4)")
            .bind(space).bind(registry).bind(public.space_id).bind(serde_json::to_value(&command).unwrap()).execute(&pool).await.unwrap();
        let dossier = Uuid::new_v4();
        sqlx::query("INSERT INTO task_dossiers(id,space_id,task_key,tracker_instance_id,task_id) VALUES($1,$2,'SAME-1',$3,$4)")
            .bind(dossier).bind(space).bind(task.tracker_instance_id).bind(task.task_id).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO managed_task_document_links(id,space_id,dossier_id,document_id,revision_id,created_by,registry_instance_id,namespace_id) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
            .bind(Uuid::new_v4()).bind(space).bind(dossier).bind(document).bind(revision).bind(admin).bind(registry).bind(public.space_id).execute(&pool).await.unwrap();
        mappings.push((space, public));
        documents.push(document);
        revisions.push(revision);
        attachments.push(attachment);
    }
    let draft = Uuid::new_v4();
    sqlx::query("INSERT INTO documents(id,space_id,slug,title,owner_id) VALUES($1,$2,'draft-only','Draft-only title',$3)").bind(draft).bind(mappings[0].0).bind(admin).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO document_drafts(document_id,author_id,content_markdown) VALUES($1,$2,'Needle private unpublished text')").bind(draft).bind(admin).execute(&pool).await.unwrap();
    assert!(
        !sqlx::query_scalar::<_, bool>(
            "SELECT active FROM wiki_space_admission_state WHERE singleton"
        )
        .fetch_one(&pool)
        .await
        .unwrap()
    );

    // Dossiers and their document FKs are local to one Wiki space. Publication
    // metadata differs from a newer working draft without rewriting history.
    sqlx::query("UPDATE documents SET title='Needle unpublished-title',status='draft' WHERE id=$1")
        .bind(documents[0])
        .execute(&pool)
        .await
        .unwrap();
    for (kind, key, ids) in [
        ("task", "DOS-1", vec![documents[0], draft]),
        ("task", "DOS-2", vec![draft]),
        ("phase", "phase-1", vec![documents[0], draft]),
        ("phase", "phase-2", vec![draft]),
    ] {
        let dossier = Uuid::new_v4();
        let statement = if kind == "task" {
            "INSERT INTO task_dossiers(id,space_id,task_key) VALUES($1,$2,$3)"
        } else {
            "INSERT INTO phase_dossiers(id,space_id,phase_key) VALUES($1,$2,$3)"
        };
        sqlx::query(statement)
            .bind(dossier)
            .bind(mappings[0].0)
            .bind(key)
            .execute(&pool)
            .await
            .unwrap();
        let statement = if kind == "task" {
            "INSERT INTO document_task_links(space_id,document_id,task_dossier_id,created_by) VALUES($1,$2,$3,$4)"
        } else {
            "INSERT INTO document_phase_links(space_id,document_id,phase_dossier_id,created_by) VALUES($1,$2,$3,$4)"
        };
        for document in ids {
            sqlx::query(statement)
                .bind(mappings[0].0)
                .bind(document)
                .bind(dossier)
                .bind(admin)
                .execute(&pool)
                .await
                .unwrap();
        }
        let rejected = sqlx::query(statement)
            .bind(mappings[0].0)
            .bind(documents[1])
            .bind(dossier)
            .bind(admin)
            .execute(&pool)
            .await
            .unwrap_err();
        assert_eq!(
            rejected.as_database_error().unwrap().code().as_deref(),
            Some("23503")
        );
    }

    // A new common space can own Wiki documents without a Tracker project.
    let owner_identity = CentralIdentity {
        issuer: "https://auth.example.test".into(),
        subject: Uuid::new_v4().to_string(),
    };
    let now = chrono::Utc::now().timestamp();
    let authority = shared::space_owner::WikiSpaceOwnerAuthority::confirmed(
        owner_identity,
        "synthetic-owner-token-not-a-live-secret".into(),
        AdmissionValidity::confirmed(now, now + 60, i64::MAX).unwrap(),
    )
    .unwrap();
    let public = SpaceRef {
        registry_instance_id: registry,
        space_id: Uuid::new_v4(),
    };
    let mut command = shared::space_owner::SpaceOwnerCommandV2 {
        schema_version: 2,
        space: public.clone(),
        participant_instance_id: wiki_instance,
        operation_id: Uuid::new_v4(),
        revision: 1,
        desired_state: shared::space_owner::DesiredSpaceState::Active,
        metadata: pdlc_shared::space_lifecycle::SpaceMetadata {
            slug: "standalone-knowledge".into(),
            name: "База знаний без проекта".into(),
            description: "Independent Wiki".into(),
            responsible_subject: central.to_string(),
        },
    };
    let initial = backend
        .apply_space_owner_v2(&command, &authority)
        .await
        .unwrap();
    let local = initial.local_resource.as_ref().unwrap().resource_id;
    assert_ne!(local, public.space_id);
    let replay = backend
        .apply_space_owner_v2(&command, &authority)
        .await
        .unwrap();
    assert_eq!(replay, initial);
    let links: i64 =
        sqlx::query_scalar("SELECT count(*) FROM wiki_space_projects WHERE space_id=$1")
            .bind(local)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(links, 0);
    let independent = claims_at_revision(
        user,
        central,
        "user",
        &[(local, public.clone())],
        &[public.clone()],
        &[local],
        1,
    );
    let key = public.space_id.simple().to_string().to_ascii_uppercase();
    let body: CreateDocumentRequest =
        serde_json::from_value(serde_json::json!({"title":"Самостоятельный документ"})).unwrap();
    let created = backend
        .create_document(&independent, &key, body)
        .await
        .unwrap();
    let mut conflict = command.clone();
    conflict.metadata.name = "Другой замысел того же ключа".into();
    assert!(
        backend
            .apply_space_owner_v2(&conflict, &authority)
            .await
            .is_err()
    );
    command.operation_id = Uuid::new_v4();
    command.revision = 2;
    command.metadata.name = "Переименованная база знаний".into();
    // Responsible metadata must not grant a new administrator role.
    command.metadata.responsible_subject = admin_central.to_string();
    let renamed = backend
        .apply_space_owner_v2(&command, &authority)
        .await
        .unwrap();
    assert_eq!(renamed.local_resource, initial.local_resource);
    let (preserved_owner, name): (Uuid, String) =
        sqlx::query_as("SELECT owner_id,name FROM spaces WHERE id=$1")
            .bind(local)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(preserved_owner, user);
    assert_eq!(name, "Переименованная база знаний");
    let role: Option<String> =
        sqlx::query_scalar("SELECT role FROM space_members WHERE space_id=$1 AND user_id=$2")
            .bind(local)
            .bind(admin)
            .fetch_optional(&pool)
            .await
            .unwrap();
    assert!(role.is_none());
    let global: String = sqlx::query_scalar("SELECT global_role FROM users WHERE id=$1")
        .bind(user)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        global, "user",
        "explicit local space role must not become a global admin"
    );
    command.operation_id = Uuid::new_v4();
    command.revision = 3;
    command.desired_state = shared::space_owner::DesiredSpaceState::Archived;
    assert!(
        backend
            .apply_space_owner_v2(&command, &authority)
            .await
            .unwrap()
            .drained
    );
    assert!(
        !backend
            .get_document(&independent, &created.id.to_string())
            .await
            .unwrap()
            .can_edit
    );
    let denied=sqlx::query("INSERT INTO documents(id,space_id,slug,title,owner_id) VALUES($1,$2,'preactivation-archive-bypass','Must not commit',$3)").bind(Uuid::new_v4()).bind(local).bind(user).execute(&pool).await.unwrap_err();
    assert_eq!(
        denied.as_database_error().unwrap().code().as_deref(),
        Some("42501"),
        "archive is effective before global activation"
    );
    assert!(
        sqlx::query("UPDATE spaces SET name='Unowned edit' WHERE id=$1")
            .bind(local)
            .execute(&pool)
            .await
            .is_err()
    );
    command.operation_id = Uuid::new_v4();
    command.revision = 4;
    command.desired_state = shared::space_owner::DesiredSpaceState::Active;
    let restored = backend
        .apply_space_owner_v2(&command, &authority)
        .await
        .unwrap();
    assert_eq!(restored.local_resource, initial.local_resource);
    assert!(
        backend
            .get_document(&independent, &created.id.to_string())
            .await
            .unwrap()
            .can_edit
    );
    let receipts:i64=sqlx::query_scalar("SELECT count(*) FROM wiki_space_owner_operations WHERE registry_instance_id=$1 AND space_id=$2").bind(registry).bind(public.space_id).fetch_one(&pool).await.unwrap();
    assert_eq!(
        receipts, 4,
        "replay must not create a duplicate receipt or space"
    );

    // M:N projection imports explicit legacy IDs and preserves row identity/history.
    let projection_tracker: Uuid = std::env::var("WIKI_NAMESPACE__TRACKER_INSTANCE_ID")
        .unwrap()
        .parse()
        .unwrap();
    let legacy_project = pdlc_shared::resource_context::NamespaceRef {
        registry_instance_id: registry,
        namespace_id: Uuid::new_v4(),
    };
    let project_ref = pdlc_shared::space_access::ProjectRef {
        tracker_instance_id: projection_tracker,
        project_id: Uuid::new_v4(),
    };
    let mut original_link_ids = Vec::new();
    for (local, _) in &mappings {
        let id:Uuid=sqlx::query_scalar("INSERT INTO wiki_space_projects(space_id,registry_instance_id,namespace_id,created_by) VALUES($1,$2,$3,$4) RETURNING id").bind(local).bind(registry).bind(legacy_project.namespace_id).bind(admin).fetch_one(&pool).await.unwrap();
        original_link_ids.push(id);
    }
    let projection_authority = pdlc_shared::space_lifecycle::SpaceOwnerAuthority::confirmed(
        CentralIdentity {
            issuer: "https://auth.example.test".into(),
            subject: "registered-tracker".into(),
        },
        AdmissionValidity::confirmed(
            chrono::Utc::now().timestamp(),
            chrono::Utc::now().timestamp() + 60,
            i64::MAX,
        )
        .unwrap(),
    )
    .unwrap();
    let mut projection = pdlc_shared::project_spaces::ProjectSpacesProjection {
        schema_version: 1,
        operation_id: Uuid::new_v4(),
        project: project_ref.clone(),
        revision: 1,
        spaces: vec![mappings[0].1.clone()],
        legacy_project: Some(legacy_project.clone()),
    };
    assert!(
        backend
            .apply_project_spaces_projection(&projection, &projection_authority)
            .await
            .is_err(),
        "unresolved B must be imported into the canonical union, never discarded"
    );
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM wiki_space_projects WHERE id=ANY($1) AND project_id IS NULL",
    )
    .bind(&original_link_ids)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 2);
    projection.spaces.push(mappings[1].1.clone());
    let mut stale_legacy = pool.begin().await.unwrap();
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut *stale_legacy)
        .await
        .unwrap();
    let old_owner: bool = sqlx::query_scalar(
        "SELECT tracker_owned FROM wiki_project_projection_state WHERE singleton",
    )
    .fetch_one(&mut *stale_legacy)
    .await
    .unwrap();
    assert!(!old_owner);
    let first_ack = backend
        .apply_project_spaces_projection(&projection, &projection_authority)
        .await
        .unwrap();
    assert!(first_ack.matches(&projection));
    let old_ui = claims(
        admin,
        admin_central,
        "admin",
        &mappings,
        &[mappings[0].1.clone()],
        &[],
    );
    assert!(
        matches!(backend.linked_projects(&old_ui,"QA01").await,Err(shared::AppError::InvalidInput(code)) if code=="project_catalog_v2_required"),
        "legacy cannot guess a ProjectRef or decode nullable old IDs after owner handoff"
    );
    assert!(
        matches!(backend.list_projects(&old_ui,None,10,0,"").await,Err(shared::AppError::InvalidInput(code)) if code=="project_catalog_v2_required")
    );

    let stale_insert=sqlx::query("INSERT INTO wiki_space_projects(space_id,registry_instance_id,namespace_id) VALUES($1,$2,$3)").bind(mappings[0].0).bind(registry).bind(Uuid::new_v4()).execute(&mut *stale_legacy).await.unwrap_err();
    assert_eq!(
        stale_insert.as_database_error().unwrap().code().as_deref(),
        Some("40001"),
        "old snapshot cannot keep the previous association owner"
    );
    stale_legacy.rollback().await.unwrap();

    assert_eq!(
        backend
            .apply_project_spaces_projection(&projection, &projection_authority)
            .await
            .unwrap(),
        first_ack,
        "lost HTTP ACK is safe to retry"
    );
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM wiki_space_projects WHERE id=ANY($1) AND project_id=$2 AND created_by=$3").bind(&original_link_ids).bind(project_ref.project_id).bind(admin).fetch_one(&pool).await.unwrap();
    assert_eq!(count, 2, "legacy link IDs/authorship are preserved");
    let first_projection = projection.clone();
    let mut conflict = projection.clone();
    conflict.spaces.pop();
    assert!(
        backend
            .apply_project_spaces_projection(&conflict, &projection_authority)
            .await
            .is_err()
    );
    // Association metadata can be projected while A is archived; content remains intact.
    let mut metadata_tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL pdlc.wiki.lifecycle_owner='admin_v2'")
        .execute(&mut *metadata_tx)
        .await
        .unwrap();
    sqlx::query("UPDATE wiki_space_catalog SET state='archived' WHERE local_space_id=$1")
        .bind(mappings[0].0)
        .execute(&mut *metadata_tx)
        .await
        .unwrap();
    metadata_tx.commit().await.unwrap();
    projection.operation_id = Uuid::new_v4();
    projection.revision = 2;
    backend
        .apply_project_spaces_projection(&projection, &projection_authority)
        .await
        .unwrap();
    projection.operation_id = Uuid::new_v4();
    projection.revision = 3;
    projection.spaces = vec![mappings[1].1.clone()];
    backend
        .apply_project_spaces_projection(&projection, &projection_authority)
        .await
        .unwrap();
    assert_eq!(
        backend
            .apply_project_spaces_projection(&first_projection, &projection_authority)
            .await
            .unwrap(),
        first_ack,
        "old receipt must not revert the current set"
    );
    let linked: Vec<Uuid> = sqlx::query_scalar(
        "SELECT space_id FROM wiki_space_projects WHERE tracker_instance_id=$1 AND project_id=$2",
    )
    .bind(projection_tracker)
    .bind(project_ref.project_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(linked, vec![mappings[1].0]);
    assert!(
        sqlx::query("DELETE FROM wiki_space_projects WHERE project_id=$1")
            .bind(project_ref.project_id)
            .execute(&pool)
            .await
            .is_err(),
        "legacy writer cannot compete with Tracker before global activation"
    );
    let preserved: i64 = sqlx::query_scalar("SELECT count(*) FROM documents WHERE id=ANY($1)")
        .bind(&documents)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(preserved, 2, "detaching A never deletes its documents");
    let mut invalid = projection.clone();
    invalid.operation_id = Uuid::new_v4();
    invalid.revision = 4;
    invalid.spaces.push(SpaceRef {
        registry_instance_id: registry,
        space_id: Uuid::new_v4(),
    });
    assert!(
        backend
            .apply_project_spaces_projection(&invalid, &projection_authority)
            .await
            .is_err()
    );
    let head:i64=sqlx::query_scalar("SELECT revision FROM wiki_project_space_projection_sets WHERE tracker_instance_id=$1 AND project_id=$2").bind(projection_tracker).bind(project_ref.project_id).fetch_one(&pool).await.unwrap();
    assert_eq!(head, 3, "partial projection failure is atomic");
    let mut metadata_tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL pdlc.wiki.lifecycle_owner='admin_v2'")
        .execute(&mut *metadata_tx)
        .await
        .unwrap();
    sqlx::query("UPDATE wiki_space_catalog SET state='active' WHERE local_space_id=$1")
        .bind(mappings[0].0)
        .execute(&mut *metadata_tx)
        .await
        .unwrap();
    metadata_tx.commit().await.unwrap();
    // A space can have no project. Admission activation still requires confirmed catalog/cohort metadata.
    sqlx::query("UPDATE wiki_space_admission_state SET active=true,cohort_revision='synthetic-native-fixture',role_migration_confirmed=true").execute(&pool).await.unwrap();
    let selected = [mappings[0].1.clone()];
    for (id, role, subject) in [(user, "user", central), (admin, "admin", admin_central)] {
        let actor = claims(id, subject, role, &mappings, &selected, &[]);
        let spaces = backend.list_spaces(&actor).await.unwrap();
        assert_eq!(spaces.spaces.len(), 1);
        assert_eq!(spaces.spaces[0].key, "QA01");
        let contexts = backend
            .namespace_contexts(&actor, None, 1, 0)
            .await
            .unwrap();
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].resource_key, "QA01");
        assert!(
            backend
                .namespace_contexts(&actor, None, 1, 1)
                .await
                .unwrap()
                .is_empty()
        );
        let hidden = pdlc_shared::resource_context::NamespaceRef {
            registry_instance_id: registry,
            namespace_id: mappings[1].1.space_id,
        };
        assert!(
            backend
                .namespace_contexts(&actor, Some(&hidden), 1, 0)
                .await
                .unwrap()
                .is_empty()
        );
        let links = backend.task_revision_links(&actor, &task).await.unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0]["space_key"], "QA01");
        assert!(
            backend.list_projects(&actor, None, 1, 0, "").await.is_err(),
            "v1 project catalog cannot silently choose a project for M:N"
        );
        assert!(
            backend
                .namespace_task_revision_links(&hidden, &task)
                .await
                .is_err(),
            "v1 machine route lacks an original actor grant"
        );
        let document = backend
            .get_document(&actor, &documents[0].to_string())
            .await
            .unwrap();
        assert!(document.body_markdown.contains("QA01"));
        assert!(matches!(
            backend
                .get_document(&actor, &documents[1].to_string())
                .await,
            Err(shared::AppError::Forbidden)
        ));
        assert!(matches!(
            backend
                .get_document_revision(&actor, &documents[1].to_string(), &revisions[1].to_string())
                .await,
            Err(shared::AppError::Forbidden)
        ));
        let query: SearchQuery =
            serde_json::from_value(serde_json::json!({"q":"Needle","limit":1})).unwrap();
        let found = backend.search(&actor, query).await.unwrap();
        assert!(!found.results.is_empty());
        assert!(
            found
                .results
                .iter()
                .all(|result| result.space_key == "QA01")
        );
        let query: EvidenceQuery =
            serde_json::from_value(serde_json::json!({"q":"Needle","limit":1})).unwrap();
        let found = backend.list_evidence(Some(&actor), query).await.unwrap();
        assert_eq!(found.evidence.len(), 1);
        assert_eq!(found.evidence[0].space_key, "QA01");
        assert_eq!(
            backend
                .download_attachment(&actor, &attachments[0].to_string())
                .await
                .unwrap()
                .bytes,
            b"synthetic attachment"
        );
        assert!(matches!(
            backend
                .download_attachment(&actor, &attachments[1].to_string())
                .await,
            Err(shared::AppError::Forbidden)
        ));
        if role == "user" {
            assert!(!document.can_edit);
            assert!(document.draft_markdown.is_empty());
            let query: SearchQuery =
                serde_json::from_value(serde_json::json!({"q":"unpublished"})).unwrap();
            assert!(
                backend
                    .search(&actor, query)
                    .await
                    .unwrap()
                    .results
                    .is_empty()
            );
            let body: CreateDocumentRequest =
                serde_json::from_value(serde_json::json!({"title":"Forbidden write"})).unwrap();
            assert!(matches!(
                backend.create_document(&actor, "QA01", body).await,
                Err(shared::AppError::Forbidden)
            ));
        }
    }
    let absent = WikiClaims {
        user_id: admin.to_string(),
        session_id: None,
        request_id: None,
        admission: None,
    };
    assert!(matches!(
        backend.list_spaces(&absent).await,
        Err(shared::AppError::Forbidden)
    ));
    let empty = claims(user, central, "user", &mappings, &[], &[]);
    assert!(backend.list_spaces(&empty).await.unwrap().spaces.is_empty());
    assert!(matches!(
        backend
            .get_document(&empty, &documents[0].to_string())
            .await,
        Err(shared::AppError::Forbidden)
    ));
    let ordinary = claims(user, central, "user", &mappings, &selected, &[]);
    assert_dossier_document_visibility(&backend, &ordinary, false).await;
    assert!(
        matches!(
            backend.get_document(&ordinary, &draft.to_string()).await,
            Err(shared::AppError::NotFound(_))
        ),
        "direct unpublished document must not disclose working metadata"
    );
    sqlx::query("INSERT INTO space_members(space_id,user_id,role) VALUES($1,$2,'editor')")
        .bind(mappings[0].0)
        .bind(user)
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        !backend
            .get_document(&ordinary, &documents[0].to_string())
            .await
            .unwrap()
            .can_edit,
        "a role granted after proof requires fresh confirmation"
    );
    let editor = claims(
        user,
        central,
        "user",
        &mappings,
        &selected,
        &[mappings[0].0],
    );
    assert!(
        backend
            .get_document(&editor, &documents[0].to_string())
            .await
            .unwrap()
            .can_edit
    );
    assert_ne!(
        ordinary
            .admission
            .as_ref()
            .unwrap()
            .replay_binding()
            .unwrap(),
        editor.admission.as_ref().unwrap().replay_binding().unwrap()
    );
    let readonly_editor = claims_with_write_scope(
        user,
        central,
        "user",
        &mappings,
        &selected,
        &[mappings[0].0],
        10,
        false,
    );
    assert_dossier_document_visibility(&backend, &readonly_editor, true).await;
    let readonly_document = backend
        .get_document(&readonly_editor, &documents[0].to_string())
        .await
        .unwrap();
    assert!(!readonly_document.can_edit);
    assert_eq!(
        readonly_document.draft_markdown, "private unpublished draft",
        "service read scope preserves confirmed editor draft reading"
    );
    assert!(readonly_document.draft_checksum.is_some());
    let query: SearchQuery =
        serde_json::from_value(serde_json::json!({"q":"unpublished"})).unwrap();
    let found = backend.search(&readonly_editor, query).await.unwrap();
    assert!(
        found
            .results
            .iter()
            .any(|result| result.id == draft.to_string()),
        "draft-only search remains an editor read"
    );
    assert!(
        found
            .results
            .iter()
            .all(|result| result.space_key == "QA01")
    );
    let body: CreateDocumentRequest =
        serde_json::from_value(serde_json::json!({"title":"Read token cannot edit"})).unwrap();
    assert!(matches!(
        backend
            .create_document(&readonly_editor, "QA01", body)
            .await,
        Err(shared::AppError::Forbidden)
    ));
    sqlx::query("DELETE FROM space_members WHERE space_id=$1 AND user_id=$2")
        .bind(mappings[0].0)
        .bind(user)
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        !backend
            .get_document(&editor, &documents[0].to_string())
            .await
            .unwrap()
            .can_edit,
        "current role revocation also wins over the old ceiling"
    );
    assert_dossier_document_visibility(&backend, &readonly_editor, false).await;
    let revoked = backend
        .get_document(&readonly_editor, &documents[0].to_string())
        .await
        .unwrap();
    assert!(
        revoked.draft_markdown.is_empty(),
        "revoked editor role closes private draft read"
    );
    let query: SearchQuery =
        serde_json::from_value(serde_json::json!({"q":"unpublished"})).unwrap();
    assert!(
        backend
            .search(&readonly_editor, query)
            .await
            .unwrap()
            .results
            .is_empty()
    );

    assert!(matches!(
        backend
            .get_document(&ordinary, &documents[1].to_string())
            .await,
        Err(shared::AppError::Forbidden)
    ));
    let context_reader = claims(
        user,
        central,
        "user",
        &mappings,
        &[mappings[0].1.clone()],
        &[],
    );
    super::project_context_tests::verify(
        &mut backend,
        &context_reader,
        &independent,
        &mappings[0].1,
        &mappings[1].1,
        &public,
    )
    .await;
    super::write_tests::verify(&backend, user, central, &mappings, documents[0]).await;
    // Real SQL writer waits behind the owner barrier and cannot use its stale proof.
    let mut owner = pool.begin().await.unwrap();
    sqlx::query("SELECT singleton FROM wiki_space_admission_state WHERE singleton FOR SHARE")
        .execute(&mut *owner)
        .await
        .unwrap();
    sqlx::query("SET LOCAL pdlc.wiki.lifecycle_owner='admin_v2'")
        .execute(&mut *owner)
        .await
        .unwrap();
    sqlx::query("UPDATE wiki_space_catalog SET state='archived',admin_revision=admin_revision+1,operation_id=$2 WHERE local_space_id=$1")
        .bind(mappings[0].0).bind(Uuid::new_v4()).execute(&mut *owner).await.unwrap();
    let writer_pool = pool.clone();
    let space = mappings[0].0;
    let writer_owner = admin;
    let write = tokio::spawn(async move {
        sqlx::query("INSERT INTO documents(id,space_id,slug,title,owner_id) VALUES($1,$2,'must-not-commit','Blocked',$3)")
            .bind(Uuid::new_v4()).bind(space).bind(writer_owner).execute(&writer_pool).await
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(
        !write.is_finished(),
        "writer must wait for the uncommitted owner decision"
    );
    owner.commit().await.unwrap();
    let error = write.await.unwrap().unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("42501")
    );
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM documents WHERE slug='must-not-commit'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
    let stale_admin = claims(admin, admin_central, "admin", &mappings, &selected, &[]);
    assert!(
        !backend
            .get_document(&stale_admin, &documents[0].to_string())
            .await
            .unwrap()
            .can_edit
    );
    assert!(
        backend
            .get_document(&stale_admin, &documents[1].to_string())
            .await
            .is_err(),
        "active B never lends write or read to an A-only actor"
    );
    let active_b = claims(
        admin,
        admin_central,
        "admin",
        &mappings,
        &[mappings[1].1.clone()],
        &[],
    );
    assert!(
        backend
            .get_document(&active_b, &documents[1].to_string())
            .await
            .unwrap()
            .can_edit
    );
    storage.delete("QA01").await.unwrap();
    storage.delete("QA02").await.unwrap();
    backend.pool.close().await;
    pool.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&root)
        .await
        .unwrap();
    root.close().await;
    tokio::fs::remove_dir(&storage_root).await.unwrap();
}

#[tokio::test]
#[ignore = "requires owned wiki_space_test PostgreSQL database"]
async fn database_guards_keep_public_status_and_hide_private_diagnostics() {
    let url = std::env::var("WIKI_TEST_DATABASE_URL").expect("owned PostgreSQL URL");
    let pool = PgPool::connect(&url).await.unwrap();
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(database, "wiki_space_test", "refuse working database");
    for (code, status) in [
        ("42501", 403),
        ("40001", 409),
        ("40P01", 409),
        ("23505", 409),
        ("XX000", 500),
    ] {
        // PostgreSQL constructs the real driver error, including code and private detail.
        let sql = format!(
            "DO $$ BEGIN RAISE EXCEPTION 'private fixture diagnostic' USING ERRCODE='{code}', DETAIL='private fixture detail'; END $$"
        );
        let error = sqlx::query(&sql).execute(&pool).await.unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some(code)
        );
        let mapped = database_error(error);
        match status {
            403 => assert!(matches!(mapped, shared::AppError::Forbidden)),
            409 => assert!(matches!(mapped, shared::AppError::Conflict(_))),
            500 => assert!(matches!(mapped, shared::AppError::Database(_))),
            _ => unreachable!(),
        }
        if status < 500 {
            assert!(!mapped.to_string().contains("private fixture"));
        }
    }
    pool.close().await;
}
