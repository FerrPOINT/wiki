//! A complete Tracker snapshot owns presentation only; Wiki bindings and ACLs own content.
use super::{PostgresWikiBackend, namespace::validate_projection};
use shared::{AppError, resource_context::*, wiki_contract::*};
use sqlx::Row;
use uuid::Uuid;

pub(super) fn validate_project(
    item: &ResourceContextSummary,
    registry: Uuid,
    instance: Uuid,
) -> Result<(), AppError> {
    let b = &item.binding;
    let command = OwnerCommand {
        schema_version: b.schema_version,
        namespace: b.namespace.clone(),
        resource: b.resource.clone(),
        operation_id: b.operation_id,
        generation: b.generation,
        state: b.state.clone(),
        create_spec: None,
    };
    if !command.valid_for(ResourceKind::TrackerProject, instance, registry)
        || !b.matches(&command)
        || item.label.trim().is_empty()
        || item.resource_key.trim().is_empty()
    {
        return Err(AppError::Unavailable(
            "invalid_tracker_project_readback".into(),
        ));
    }
    Ok(())
}

impl PostgresWikiBackend {
    pub(super) async fn project_catalog_actor(
        &self,
        claims: &WikiClaims,
    ) -> Result<Uuid, AppError> {
        let id = super::mapping::parse_uuid(&claims.user_id, "user")?;
        let actor =
            sqlx::query("SELECT global_role,central_sub FROM users WHERE id=$1 AND is_active=true")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(AppError::database)?
                .ok_or(AppError::Unauthorized)?;
        // Project metadata visibility is separate from permission to view a Wiki space.
        if actor.get::<String, _>("global_role") != "admin"
            && actor
                .get::<Option<String>, _>("central_sub")
                .filter(|sub| !sub.trim().is_empty())
                .is_none()
        {
            return Err(AppError::Forbidden);
        }
        Ok(id)
    }
    pub(super) async fn refresh_projects(&self) -> Result<(), AppError> {
        let Some(reader) = &self.task_reader else {
            return Ok(());
        };
        let result = self.replace_projects(reader.projects().await).await;
        if result.is_err() {
            sqlx::query("UPDATE wiki_workspace_catalog_sync SET available=false WHERE id=true")
                .execute(&self.pool)
                .await
                .map_err(AppError::database)?;
        }
        result
    }

    pub(super) async fn replace_projects(
        &self,
        source: Result<Vec<ResourceContextSummary>, AppError>,
    ) -> Result<(), AppError> {
        let projects = source?;
        let mut tx = self.pool.begin().await.map_err(AppError::database)?;
        sqlx::query("UPDATE wiki_workspace_projects SET present=false")
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?;
        for item in projects {
            let b = &item.binding;
            let affected = sqlx::query("INSERT INTO wiki_workspace_projects(registry_instance_id,namespace_id,tracker_instance_id,tracker_project_id,generation,presentation,name,project_key) VALUES($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT(registry_instance_id,namespace_id) DO UPDATE SET generation=EXCLUDED.generation,presentation=EXCLUDED.presentation,name=EXCLUDED.name,project_key=EXCLUDED.project_key,present=true,observed_at=now() WHERE wiki_workspace_projects.tracker_instance_id=EXCLUDED.tracker_instance_id AND wiki_workspace_projects.tracker_project_id=EXCLUDED.tracker_project_id AND wiki_workspace_projects.generation<=EXCLUDED.generation")
                .bind(b.namespace.registry_instance_id).bind(b.namespace.namespace_id).bind(b.resource.instance_id).bind(b.resource.resource_id).bind(b.generation).bind(serde_json::to_value(&item).map_err(AppError::internal)?).bind(&item.label).bind(&item.resource_key)
                .execute(&mut *tx).await.map_err(AppError::database)?.rows_affected();
            if affected != 1 {
                return Err(AppError::Unavailable(
                    "tracker_projection_identity_or_generation_conflict".into(),
                ));
            }
        }
        sqlx::query("INSERT INTO wiki_workspace_catalog_sync(id,available) VALUES(true,true) ON CONFLICT(id) DO UPDATE SET available=true,observed_at=now()")
            .execute(&mut *tx).await.map_err(AppError::database)?;
        tx.commit().await.map_err(AppError::database)
    }

    pub(super) async fn list_projects(
        &self,
        claims: &WikiClaims,
        namespace: Option<&NamespaceRef>,
        limit: i64,
        offset: i64,
        search: &str,
    ) -> Result<WorkspaceProjectPage, AppError> {
        // The legacy namespace catalog identifies exactly one Tracker project per
        // namespace; it cannot represent a canonical M:N ProjectRef set.
        self.require_legacy_project_context().await?;
        self.project_catalog_actor(claims).await?;
        if !(1..=100).contains(&limit) || offset < 0 {
            return Err(AppError::invalid_input("invalid_project_page"));
        }
        if !sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM wiki_workspace_catalog_sync)",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::database)?
        {
            return Err(AppError::Unavailable(
                "tracker_project_catalog_unavailable".into(),
            ));
        }
        let registry = std::env::var("WIKI_NAMESPACE__REGISTRY_INSTANCE_ID")
            .ok()
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| AppError::Unavailable("namespace_owner_not_configured".into()))?;
        let instance = std::env::var("WIKI_NAMESPACE__TRACKER_INSTANCE_ID")
            .ok()
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| AppError::Unavailable("tracker_reader_not_configured".into()))?;
        let rows = sqlx::query("SELECT w.presentation,w.observed_at::text AS observed_at,(w.observed_at<now()-interval '5 minutes' OR NOT coalesce((SELECT available FROM wiki_workspace_catalog_sync),false)) AS stale,b.command AS wiki_command,s.id AS space_id,s.key AS space_key,(u.global_role='admin' OR sm.role IN ('admin','editor','viewer')) AS allowed FROM wiki_workspace_projects w JOIN users u ON u.id=$1 AND u.is_active LEFT JOIN wiki_namespace_bindings b ON b.registry_instance_id=w.registry_instance_id AND b.namespace_id=w.namespace_id LEFT JOIN spaces s ON s.id=b.resource_id LEFT JOIN space_members sm ON sm.space_id=s.id AND sm.user_id=u.id WHERE w.present AND ($2::uuid IS NULL OR (w.registry_instance_id=$2 AND w.namespace_id=$3)) AND position($4 in lower(w.name||' '||w.project_key))>0 ORDER BY w.name,w.registry_instance_id,w.namespace_id LIMIT $5 OFFSET $6")
            .bind(super::mapping::parse_uuid(&claims.user_id,"user")?).bind(namespace.map(|n|n.registry_instance_id)).bind(namespace.map(|n|n.namespace_id)).bind(search.trim().to_lowercase()).bind(limit).bind(offset).fetch_all(&self.pool).await.map_err(AppError::database)?;
        let total = sqlx::query_scalar("SELECT count(*) FROM wiki_workspace_projects w WHERE w.present AND ($1::uuid IS NULL OR (w.registry_instance_id=$1 AND w.namespace_id=$2)) AND position($3 in lower(w.name||' '||w.project_key))>0")
            .bind(namespace.map(|n|n.registry_instance_id)).bind(namespace.map(|n|n.namespace_id)).bind(search.trim().to_lowercase()).fetch_one(&self.pool).await.map_err(AppError::database)?;
        let mut projections = Vec::with_capacity(rows.len());
        for row in &rows {
            let item: ResourceContextSummary = serde_json::from_value(row.get("presentation"))
                .map_err(|_| AppError::Unavailable("invalid_tracker_projection".into()))?;
            validate_project(&item, registry, instance)?;
            projections.push(item);
        }
        let ids: Vec<Uuid> = projections
            .iter()
            .map(|item| item.binding.namespace.namespace_id)
            .collect();
        let mut collections = self.spaces_for_projects(claims, registry, &ids).await?;
        let mut items = Vec::with_capacity(rows.len());
        for (row, item) in rows.into_iter().zip(projections) {
            let mut project = WorkspaceProject {
                namespace: item.binding.namespace.clone(),
                tracker: item.binding.resource.clone(),
                generation: item.binding.generation,
                name: item.label,
                project_key: item.resource_key,
                state: item.binding.state,
                observed_at: row.get("observed_at"),
                stale: row.get("stale"),
                binding: None,
                space_key: None,
                resource_status: ProjectResourceStatus::NotConnected,
                spaces: Vec::new(),
            };
            if let Some(value) = row.get::<Option<serde_json::Value>, _>("wiki_command") {
                if !row.get::<Option<bool>, _>("allowed").unwrap_or(false) {
                    project.resource_status = ProjectResourceStatus::Forbidden;
                } else {
                    match serde_json::from_value::<OwnerCommand>(value) {
                        Ok(command)
                            if validate_projection(&command).is_ok()
                                && command.namespace == project.namespace
                                && Some(command.resource.resource_id)
                                    == row.get::<Option<Uuid>, _>("space_id") =>
                        {
                            project.binding = Some(command.readback(true));
                            project.space_key = row.get("space_key");
                            project.resource_status = ProjectResourceStatus::Connected;
                        }
                        _ => project.resource_status = ProjectResourceStatus::Unavailable,
                    }
                }
            }
            project.spaces = collections
                .spaces
                .remove(&project.namespace.namespace_id)
                .unwrap_or_default();
            project.resource_status = if project
                .spaces
                .iter()
                .any(|space| matches!(space.resource_status, ProjectResourceStatus::Connected))
            {
                ProjectResourceStatus::Connected
            } else if !project.spaces.is_empty() {
                ProjectResourceStatus::Unavailable
            } else if collections.denied.contains(&project.namespace.namespace_id) {
                ProjectResourceStatus::Forbidden
            } else {
                ProjectResourceStatus::NotConnected
            };
            items.push(project);
        }
        Ok(WorkspaceProjectPage { items, total })
    }

    pub(super) async fn space_context(
        &self,
        claims: &WikiClaims,
        key: &str,
    ) -> Result<Option<ResourceContextSummary>, AppError> {
        let space = self
            .ensure_space_access(claims, key, app::wiki::WikiSpaceAccess::View)
            .await?;
        let row = sqlx::query("SELECT name,key FROM spaces WHERE id=$1")
            .bind(space)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::database)?;
        Ok(self
            .namespace_binding(space)
            .await?
            .map(|binding| ResourceContextSummary {
                binding,
                label: row.get("name"),
                resource_key: row.get("key"),
            }))
    }

    pub(super) async fn document_context(
        &self,
        claims: &WikiClaims,
        id: Uuid,
    ) -> Result<Option<ResourceContextSummary>, AppError> {
        let row=sqlx::query("SELECT d.space_id,s.key,s.name FROM documents d JOIN spaces s ON s.id=d.space_id WHERE d.id=$1").bind(id).fetch_optional(&self.pool).await.map_err(AppError::database)?.ok_or_else(||AppError::not_found("document",id))?;
        let space = row.get::<Uuid, _>("space_id");
        self.ensure_space_id_access(claims, space, app::wiki::WikiSpaceAccess::View)
            .await?;
        Ok(self
            .namespace_binding(space)
            .await?
            .map(|binding| ResourceContextSummary {
                binding,
                label: row.get("name"),
                resource_key: row.get("key"),
            }))
    }
}
