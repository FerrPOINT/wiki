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
        let rows=sqlx::query("SELECT s.id,s.key,s.name FROM spaces s WHERE s.archived_at IS NULL AND NOT EXISTS(SELECT 1 FROM wiki_namespace_bindings b WHERE b.resource_id=s.id) ORDER BY s.key,s.id LIMIT $1 OFFSET $2").bind(limit.clamp(1,100)).bind(offset.max(0)).fetch_all(&self.pool).await.map_err(AppError::database)?;
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
            let exists = sqlx::query("SELECT id FROM spaces WHERE id=$1 FOR UPDATE")
                .bind(command.resource.resource_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(AppError::database)?
                .is_some();
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
