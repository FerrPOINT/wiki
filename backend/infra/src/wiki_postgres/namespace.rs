//! Namespace projection belongs to Wiki; it does not mirror Tracker tasks.
use super::PostgresWikiBackend;
use shared::{AppError, resource_context::*};
use sqlx::Row;
use uuid::Uuid;

pub fn registered_machine(subject: &str) -> bool {
    [
        "WIKI_NAMESPACE__OWNER_SUBJECTS",
        "WIKI_NAMESPACE__READER_SUBJECTS",
    ]
    .iter()
    .any(|key| {
        std::env::var(key)
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .any(|s| !s.is_empty() && s == subject)
    })
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateSpec {
    key: String,
    name: String,
    description: Option<String>,
    owner_id: Option<Uuid>,
    owner_subject: Option<String>,
}
pub(super) fn validate_projection(command: &OwnerCommand) -> Result<(), AppError> {
    let instance = std::env::var("WIKI_NAMESPACE__INSTANCE_ID")
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| AppError::Unavailable("namespace_owner_not_configured".into()))?;
    let registry = std::env::var("WIKI_NAMESPACE__REGISTRY_INSTANCE_ID")
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| AppError::Unavailable("namespace_owner_not_configured".into()))?;
    if !command.valid_for(ResourceKind::WikiSpace, instance, registry) {
        return Err(AppError::Unavailable("invalid_namespace_projection".into()));
    }
    Ok(())
}
impl PostgresWikiBackend {
    pub(super) async fn namespace_available_resources(
        &self,
        claims: &shared::WikiClaims,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ResourceCatalogItem>, AppError> {
        let instance = std::env::var("WIKI_NAMESPACE__INSTANCE_ID")
            .ok()
            .and_then(|value| value.parse::<Uuid>().ok())
            .filter(|id| !id.is_nil())
            .ok_or_else(|| AppError::Unavailable("namespace_owner_not_configured".into()))?;
        let rows=sqlx::query("SELECT s.id,s.key,s.name FROM spaces s WHERE s.archived_at IS NULL AND NOT s.namespace_managed AND NOT EXISTS(SELECT 1 FROM wiki_namespace_bindings b WHERE b.resource_id=s.id) ORDER BY s.key,s.id LIMIT $1 OFFSET $2").bind(limit.clamp(1,100)).bind(offset.max(0)).fetch_all(&self.pool).await.map_err(AppError::database)?;
        let mut result = Vec::new();
        for row in rows {
            let id: Uuid = row.get("id");
            match self
                .ensure_space_id_access(claims, id, app::wiki::WikiSpaceAccess::Admin)
                .await
            {
                Ok(_) => {}
                Err(AppError::Forbidden) => continue,
                Err(error) => return Err(error),
            }
            result.push(ResourceCatalogItem {
                resource: ResourceRef {
                    kind: ResourceKind::WikiSpace,
                    instance_id: instance,
                    resource_id: id,
                },
                label: row.get("name"),
                resource_key: row.get("key"),
            });
        }
        Ok(result)
    }
    pub(super) async fn namespace_stats(
        &self,
        claims: &shared::WikiClaims,
        namespace: &NamespaceRef,
    ) -> Result<ResourceStats, AppError> {
        let contexts = self
            .namespace_contexts(claims, Some(namespace), 1, 0)
            .await?;
        let resource = contexts
            .into_iter()
            .next()
            .ok_or_else(|| AppError::not_found("namespace_binding", namespace.namespace_id))?;
        let id = resource.binding.resource.resource_id;
        let row=sqlx::query("SELECT (SELECT count(*) FROM documents WHERE space_id=$1) AS documents,(SELECT count(*) FROM document_revisions r JOIN documents d ON d.id=r.document_id WHERE d.space_id=$1) AS revisions").bind(id).fetch_one(&self.pool).await.map_err(AppError::database)?;
        Ok(ResourceStats {
            binding: resource.binding,
            counters: std::collections::BTreeMap::from([
                ("documents".into(), row.get("documents")),
                ("revisions".into(), row.get("revisions")),
            ]),
        })
    }
    pub(super) async fn namespace_contexts(
        &self,
        claims: &shared::WikiClaims,
        namespace: Option<&NamespaceRef>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ResourceContextSummary>, AppError> {
        let rows=sqlx::query("SELECT b.command,s.name,s.key,s.id FROM wiki_namespace_bindings b JOIN spaces s ON s.id=b.resource_id WHERE ($1::uuid IS NULL OR (b.registry_instance_id=$1 AND b.namespace_id=$2)) ORDER BY s.name,s.id LIMIT $3 OFFSET $4").bind(namespace.map(|n| n.registry_instance_id)).bind(namespace.map(|n| n.namespace_id)).bind(limit.clamp(1,100)).bind(offset.max(0)).fetch_all(&self.pool).await.map_err(AppError::database)?;
        let mut result = Vec::new();
        for row in rows {
            match self
                .ensure_space_id_access(claims, row.get("id"), app::wiki::WikiSpaceAccess::View)
                .await
            {
                Ok(_) => {}
                Err(AppError::Forbidden) => continue,
                Err(error) => return Err(error),
            }
            let command: OwnerCommand = serde_json::from_value(row.get("command"))
                .map_err(|_| AppError::Unavailable("invalid_namespace_projection".into()))?;
            validate_projection(&command)?;
            result.push(ResourceContextSummary {
                binding: command.readback(true),
                label: row.get("name"),
                resource_key: row.get("key"),
            });
        }
        Ok(result)
    }
    pub(super) async fn namespace_binding(
        &self,
        space: Uuid,
    ) -> Result<Option<OwnerReadback>, AppError> {
        let value: Option<serde_json::Value> =
            sqlx::query_scalar("SELECT command FROM wiki_namespace_bindings WHERE resource_id=$1")
                .bind(space)
                .fetch_optional(&self.pool)
                .await
                .map_err(AppError::database)?;
        if value.is_none()
            && sqlx::query_scalar::<_, bool>("SELECT namespace_managed FROM spaces WHERE id=$1")
                .bind(space)
                .fetch_optional(&self.pool)
                .await
                .map_err(AppError::database)?
                .unwrap_or(false)
        {
            return Err(AppError::Unavailable("namespace_projection_missing".into()));
        }
        value
            .map(|value| {
                let command: OwnerCommand = serde_json::from_value(value)
                    .map_err(|_| AppError::Unavailable("invalid_namespace_projection".into()))?;
                validate_projection(&command)?;
                Ok(command.readback(true))
            })
            .transpose()
    }
    pub(super) async fn apply_namespace(
        &self,
        command: &OwnerCommand,
    ) -> Result<OwnerReadback, AppError> {
        let instance = std::env::var("WIKI_NAMESPACE__INSTANCE_ID")
            .ok()
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| AppError::Unavailable("namespace_owner_not_configured".into()))?;
        let registry = std::env::var("WIKI_NAMESPACE__REGISTRY_INSTANCE_ID")
            .ok()
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| AppError::Unavailable("namespace_owner_not_configured".into()))?;
        if !command.valid_for(ResourceKind::WikiSpace, instance, registry) {
            return Err(AppError::invalid_input("invalid_namespace_owner_command"));
        }
        let mut tx = self.pool.begin().await.map_err(AppError::database)?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
            .bind(command.resource.resource_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?;
        if let Some(row) = sqlx::query(
            "SELECT command FROM wiki_namespace_bindings WHERE resource_id=$1 FOR UPDATE",
        )
        .bind(command.resource.resource_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(AppError::database)?
        {
            let old: OwnerCommand = serde_json::from_value(row.get("command"))
                .map_err(|_| AppError::Unavailable("invalid_namespace_projection".into()))?;
            if !command.follows(&old) {
                return Err(AppError::conflict("namespace_binding_conflict"));
            }
            if command == &old {
                tx.commit().await.map_err(AppError::database)?;
                return Ok(old.readback(true));
            }
        } else {
            if command.state != "active" {
                return Err(AppError::conflict("binding_required_before_lifecycle"));
            }
            let existing =
                sqlx::query("SELECT namespace_managed FROM spaces WHERE id=$1 FOR UPDATE")
                    .bind(command.resource.resource_id)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(AppError::database)?;
            if existing
                .as_ref()
                .is_some_and(|row| row.get::<bool, _>("namespace_managed"))
            {
                return Err(AppError::Unavailable("namespace_projection_missing".into()));
            }
            let exists = existing.is_some();
            if exists && command.create_spec.is_some() {
                return Err(AppError::conflict("existing_resource_requires_attach"));
            }
            if !exists {
                let spec: CreateSpec =
                    serde_json::from_value(command.create_spec.clone().ok_or_else(|| {
                        AppError::not_found("space", command.resource.resource_id)
                    })?)
                    .map_err(|_| AppError::invalid_input("invalid_space_create_spec"))?;
                let key = app::wiki::normalize_space_key(&spec.key)?;
                let owner_id = match (spec.owner_id, spec.owner_subject) {
                    (Some(id), None) => id,
                    (None, Some(subject)) => sqlx::query_scalar(
                        "SELECT id FROM users WHERE central_sub=$1 AND is_active",
                    )
                    .bind(subject)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(AppError::database)?
                    .ok_or_else(|| AppError::conflict("responsible_wiki_profile_required"))?,
                    _ => return Err(AppError::invalid_input("one_resource_owner_required")),
                };
                let active: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM users WHERE id=$1 AND is_active)",
                )
                .bind(owner_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(AppError::database)?;
                if !active {
                    return Err(AppError::conflict(
                        "active_responsible_wiki_profile_required",
                    ));
                }
                if spec.name.trim().is_empty() || spec.name.chars().count() > 200 {
                    return Err(AppError::invalid_input("invalid_space_name"));
                }
                sqlx::query("INSERT INTO spaces(id,key,name,description,owner_id,created_at,updated_at) VALUES($1,$2,$3,$4,$5,now(),now())").bind(command.resource.resource_id).bind(key).bind(spec.name).bind(spec.description.unwrap_or_default()).bind(owner_id).execute(&mut *tx).await.map_err(AppError::database)?;
                sqlx::query("INSERT INTO space_members(space_id,user_id,role,joined_at) VALUES($1,$2,'admin',now())").bind(command.resource.resource_id).bind(owner_id).execute(&mut *tx).await.map_err(AppError::database)?;
            }
        }
        sqlx::query("INSERT INTO wiki_namespace_bindings(resource_id,registry_instance_id,namespace_id,generation,state,command) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(resource_id) DO UPDATE SET generation=EXCLUDED.generation,state=EXCLUDED.state,command=EXCLUDED.command")
            .bind(command.resource.resource_id).bind(command.namespace.registry_instance_id).bind(command.namespace.namespace_id).bind(command.generation).bind(&command.state).bind(serde_json::to_value(command).map_err(|e| AppError::Internal(e.to_string()))?).execute(&mut *tx).await.map_err(|error| { tracing::warn!(error=%error,"namespace owner write rejected"); AppError::conflict("namespace_binding_rejected") })?;
        tx.commit().await.map_err(AppError::database)?;
        Ok(command.readback(true))
    }
}

#[cfg(test)]
mod namespace_postgres_tests {
    use super::*;
    use shared::wiki_contract::{WikiClaims, WikiSettingsSnapshot};

    async fn backend() -> (PostgresWikiBackend, Uuid) {
        let url = std::env::var("WIKI_TEST_DATABASE_URL").expect("disposable QA database required");
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        sqlx::migrate::Migrator::new(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../migrations"),
        )
        .await
        .unwrap()
        .run(&pool)
        .await
        .unwrap();
        let owner = Uuid::now_v7();
        sqlx::query("INSERT INTO users(id,email,username,display_name,password_hash) VALUES($1,$2,$3,'Namespace QA','unused')")
            .bind(owner).bind(format!("{owner}@example.test")).bind(format!("qa-{owner}"))
            .execute(&pool).await.unwrap();
        let config = shared::AppConfig::default();
        (
            PostgresWikiBackend {
                pool,
                task_reader: None,
                auth: config.auth.clone(),
                storage: std::sync::Arc::new(crate::LocalWikiAttachmentStorage::new(
                    std::env::temp_dir().join(format!("wiki-namespace-{owner}")),
                )),
                max_upload_bytes: config.storage.max_upload_bytes,
                staged_attachment_ttl_hours: config.maintenance.staged_attachment_ttl_hours,
                maintenance_batch_size: config.maintenance.batch_size,
                settings: WikiSettingsSnapshot::from_config(&config),
            },
            owner,
        )
    }

    fn create_command(owner: Uuid) -> OwnerCommand {
        OwnerCommand {
            schema_version: 1,
            namespace: NamespaceRef {
                registry_instance_id: std::env::var("WIKI_NAMESPACE__REGISTRY_INSTANCE_ID")
                    .expect("QA registry identity required")
                    .parse()
                    .unwrap(),
                namespace_id: Uuid::now_v7(),
            },
            resource: ResourceRef {
                kind: ResourceKind::WikiSpace,
                instance_id: std::env::var("WIKI_NAMESPACE__INSTANCE_ID")
                    .expect("QA Wiki identity required")
                    .parse()
                    .unwrap(),
                resource_id: Uuid::now_v7(),
            },
            operation_id: Uuid::now_v7(),
            generation: 1,
            state: "active".into(),
            create_spec: Some(serde_json::json!({
                "key": format!("QA{}", &owner.simple().to_string()[..24]),
                "name": "Namespace QA", "owner_id": owner,
            })),
        }
    }

    #[tokio::test]
    #[ignore = "requires disposable PostgreSQL and QA Wiki/registry instance IDs"]
    async fn lifecycle_fences_replays_and_legacy_writes() {
        let (backend, owner) = backend().await;
        let command = create_command(owner);
        let space = command.resource.resource_id;
        assert!(
            backend
                .apply_namespace(&command)
                .await
                .unwrap()
                .matches(&command)
        );
        assert!(
            backend
                .apply_namespace(&command)
                .await
                .unwrap()
                .matches(&command)
        );
        let marker: bool = sqlx::query_scalar("SELECT namespace_managed FROM spaces WHERE id=$1")
            .bind(space)
            .fetch_one(&backend.pool)
            .await
            .unwrap();
        assert!(marker);
        let mut conflict = command.clone();
        conflict.operation_id = Uuid::now_v7();
        assert!(matches!(
            backend.apply_namespace(&conflict).await,
            Err(AppError::Conflict(_))
        ));
        conflict.generation = 2;
        conflict.create_spec = None;
        conflict.namespace.namespace_id = Uuid::now_v7();
        assert!(matches!(
            backend.apply_namespace(&conflict).await,
            Err(AppError::Conflict(_))
        ));

        let document = Uuid::now_v7();
        sqlx::query("INSERT INTO documents(id,space_id,slug,title,owner_id) VALUES($1,$2,'qa','QA document',$3)")
            .bind(document).bind(space).bind(owner).execute(&backend.pool).await.unwrap();
        let revision = Uuid::now_v7();
        sqlx::query("INSERT INTO document_revisions(id,document_id,version,title,content_markdown,content_html,content_text,content_checksum,author_id) VALUES($1,$2,1,'QA revision','body','body','body','qa',$3)")
            .bind(revision).bind(document).bind(owner).execute(&backend.pool).await.unwrap();
        for query in [
            "UPDATE document_revisions SET title='changed' WHERE id=$1",
            "DELETE FROM document_revisions WHERE id=$1",
        ] {
            let error = sqlx::query(query)
                .bind(revision)
                .execute(&backend.pool)
                .await
                .unwrap_err();
            assert_eq!(
                error.as_database_error().unwrap().code().as_deref(),
                Some("42501")
            );
        }
        sqlx::query("UPDATE documents SET status='archived',archived_at=now() WHERE id=$1")
            .bind(document)
            .execute(&backend.pool)
            .await
            .unwrap();
        let mut archived = command.clone();
        archived.operation_id = Uuid::now_v7();
        archived.generation = 2;
        archived.state = "archived".into();
        archived.create_spec = None;
        assert!(
            backend
                .apply_namespace(&archived)
                .await
                .unwrap()
                .matches(&archived)
        );
        let error = sqlx::query("UPDATE documents SET title='changed' WHERE id=$1")
            .bind(document)
            .execute(&backend.pool)
            .await
            .unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("42501")
        );
        let error = sqlx::query("DELETE FROM spaces WHERE id=$1")
            .bind(space)
            .execute(&backend.pool)
            .await
            .unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("42501")
        );
        let mut restored = archived.clone();
        restored.operation_id = Uuid::now_v7();
        restored.generation = 3;
        restored.state = "active".into();
        assert!(
            backend
                .apply_namespace(&restored)
                .await
                .unwrap()
                .matches(&restored)
        );
        assert!(matches!(
            backend.apply_namespace(&archived).await,
            Err(AppError::Conflict(_))
        ));
        let status: String = sqlx::query_scalar("SELECT status FROM documents WHERE id=$1")
            .bind(document)
            .fetch_one(&backend.pool)
            .await
            .unwrap();
        assert_eq!(status, "archived");
        let mut tx = backend.pool.begin().await.unwrap();
        assert!(matches!(
            super::super::ensure_document_accepts_writes_tx(&mut tx, document).await,
            Err(AppError::InvalidInput(_))
        ));
        tx.rollback().await.unwrap();
        let title: String = sqlx::query_scalar("SELECT title FROM document_revisions WHERE id=$1")
            .bind(revision)
            .fetch_one(&backend.pool)
            .await
            .unwrap();
        assert_eq!(title, "QA revision");
        backend.pool.close().await;
    }

    #[tokio::test]
    #[ignore = "requires disposable PostgreSQL and QA Wiki/registry instance IDs"]
    async fn lost_projection_fails_closed_for_reads_and_legacy_writes() {
        let (backend, owner) = backend().await;
        let command = create_command(owner);
        let space = command.resource.resource_id;
        backend.apply_namespace(&command).await.unwrap();
        sqlx::query("DELETE FROM wiki_namespace_bindings WHERE resource_id=$1")
            .bind(space)
            .execute(&backend.pool)
            .await
            .unwrap();
        assert!(matches!(
            backend.namespace_binding(space).await,
            Err(AppError::Unavailable(_))
        ));
        let error = sqlx::query("UPDATE spaces SET name='changed' WHERE id=$1")
            .bind(space)
            .execute(&backend.pool)
            .await
            .unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("42501")
        );
        let error = sqlx::query("UPDATE spaces SET namespace_managed=false WHERE id=$1")
            .bind(space)
            .execute(&backend.pool)
            .await
            .unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("42501")
        );
        let claims = WikiClaims {
            user_id: owner.to_string(),
            session_id: None,
            request_id: None,
        };
        let resources = backend
            .namespace_available_resources(&claims, 100, 0)
            .await
            .unwrap();
        let advertised = resources
            .iter()
            .any(|item| item.resource.resource_id == space);
        let mut transferred = command.clone();
        transferred.namespace.namespace_id = Uuid::now_v7();
        transferred.operation_id = Uuid::now_v7();
        transferred.generation = 2;
        transferred.create_spec = None;
        let accepted = backend.apply_namespace(&transferred).await.is_ok();
        assert_eq!(
            (advertised, accepted),
            (false, false),
            "lost projection must neither advertise nor transfer an already managed resource"
        );
        backend.pool.close().await;
    }
}
