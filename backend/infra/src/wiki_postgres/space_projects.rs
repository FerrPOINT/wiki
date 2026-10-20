//! Associations never transfer space ownership or grant document permissions.
use super::database_error;
use super::{
    PostgresWikiBackend, namespace::validate_projection, workspace_catalog::validate_project,
};
use app::wiki::WikiSpaceAccess;
use shared::{AppError, resource_context::*, wiki_contract::*};
use sqlx::Row;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub(super) struct ProjectSpaces {
    pub spaces: BTreeMap<Uuid, Vec<WorkspaceSpace>>,
    pub denied: BTreeSet<Uuid>,
}

impl PostgresWikiBackend {
    /// A legacy NamespaceRef response cannot encode canonical ProjectRef-only rows.
    pub(super) async fn require_legacy_project_context(&self) -> Result<(), AppError> {
        let owned:bool=sqlx::query_scalar("SELECT tracker_owned OR (SELECT active FROM wiki_space_admission_state WHERE singleton) FROM wiki_project_projection_state WHERE singleton").fetch_one(&self.pool).await.map_err(database_error)?;
        if owned {
            return Err(AppError::invalid_input("project_catalog_v2_required"));
        }
        Ok(())
    }
    pub(super) async fn spaces_for_projects(
        &self,
        claims: &WikiClaims,
        registry: Uuid,
        projects: &[Uuid],
    ) -> Result<ProjectSpaces, AppError> {
        self.require_legacy_project_context().await?;
        let rows = sqlx::query(
            "SELECT l.namespace_id,s.id,s.key,s.name,s.namespace_managed,b.command,
             CASE WHEN s.archived_at IS NULL THEN 'active' ELSE 'archived' END AS status,
             (u.global_role='admin' OR sm.role IN ('admin','editor','viewer')) AS allowed
             FROM wiki_space_projects l JOIN spaces s ON s.id=l.space_id
             JOIN users u ON u.id=$1 AND u.is_active
             LEFT JOIN space_members sm ON sm.space_id=s.id AND sm.user_id=u.id
             LEFT JOIN wiki_namespace_bindings b ON b.resource_id=s.id
             WHERE l.registry_instance_id=$2 AND l.namespace_id=ANY($3)
             ORDER BY l.namespace_id,s.name,s.id",
        )
        .bind(super::mapping::parse_uuid(&claims.user_id, "user")?)
        .bind(registry)
        .bind(projects)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;
        let mut result = ProjectSpaces {
            spaces: BTreeMap::new(),
            denied: BTreeSet::new(),
        };
        for row in rows {
            let project = row.get::<Uuid, _>("namespace_id");
            if !row.get::<Option<bool>, _>("allowed").unwrap_or(false) {
                result.denied.insert(project);
                continue;
            }
            let receipt = row.get::<Option<serde_json::Value>, _>("command");
            let valid = match receipt {
                Some(value) => serde_json::from_value::<OwnerCommand>(value)
                    .ok()
                    .is_some_and(|command| {
                        validate_projection(&command).is_ok()
                            && command.resource.resource_id == row.get::<Uuid, _>("id")
                    }),
                None => !row.get::<bool, _>("namespace_managed"),
            };
            result
                .spaces
                .entry(project)
                .or_default()
                .push(WorkspaceSpace {
                    id: row.get("id"),
                    key: row.get("key"),
                    name: row.get("name"),
                    status: row.get("status"),
                    resource_status: if valid {
                        ProjectResourceStatus::Connected
                    } else {
                        ProjectResourceStatus::Unavailable
                    },
                });
        }
        Ok(result)
    }
    pub(super) async fn linked_projects(
        &self,
        claims: &WikiClaims,
        key: &str,
    ) -> Result<SpaceProjectsResponse, AppError> {
        let space_id = self
            .ensure_space_access(claims, key, WikiSpaceAccess::View)
            .await?;
        self.require_legacy_project_context().await?;
        self.namespace_binding(space_id).await?;
        let metadata_visible = match self.project_catalog_actor(claims).await {
            Ok(_) => true,
            Err(AppError::Forbidden) => false,
            Err(error) => return Err(error),
        };
        let rows = sqlx::query(
            "SELECT l.registry_instance_id,l.namespace_id,w.name,w.project_key,w.presentation,w.present,
             coalesce(w.present AND w.observed_at>now()-interval '5 minutes'
             AND (SELECT available FROM wiki_workspace_catalog_sync),false) AS available
             FROM wiki_space_projects l LEFT JOIN wiki_workspace_projects w
             ON (w.registry_instance_id,w.namespace_id)=(l.registry_instance_id,l.namespace_id)
             WHERE l.space_id=$1 ORDER BY w.name NULLS LAST,l.registry_instance_id,l.namespace_id",
        ).bind(space_id).fetch_all(&self.pool).await.map_err(database_error)?;
        let projects = rows
            .into_iter()
            .map(|row| {
                let presentation = row
                    .get::<Option<serde_json::Value>, _>("presentation")
                    .map(serde_json::from_value::<ResourceContextSummary>)
                    .transpose()
                    .map_err(|_| AppError::Unavailable("invalid_tracker_projection".into()))?;
                Ok(SpaceProject {
                    namespace: NamespaceRef {
                        registry_instance_id: row.get("registry_instance_id"),
                        namespace_id: row.get("namespace_id"),
                    },
                    name: if metadata_visible {
                        row.get("name")
                    } else {
                        None
                    },
                    project_key: if metadata_visible {
                        row.get("project_key")
                    } else {
                        None
                    },
                    state: presentation.map(|item| item.binding.state),
                    available: metadata_visible && row.get::<bool, _>("available"),
                })
            })
            .collect::<Result<_, AppError>>()?;
        let row = sqlx::query("SELECT id,key,name,CASE WHEN archived_at IS NULL THEN 'active' ELSE 'archived' END AS status FROM spaces WHERE id=$1")
            .bind(space_id).fetch_one(&self.pool).await.map_err(database_error)?;
        Ok(SpaceProjectsResponse {
            space: WorkspaceSpace {
                id: row.get("id"),
                key: row.get("key"),
                name: row.get("name"),
                status: row.get("status"),
                resource_status: ProjectResourceStatus::Connected,
            },
            projects,
        })
    }

    pub(super) async fn document_projects(
        &self,
        claims: &WikiClaims,
        document: Uuid,
    ) -> Result<SpaceProjectsResponse, AppError> {
        let key: String = sqlx::query_scalar(
            "SELECT s.key FROM documents d JOIN spaces s ON s.id=d.space_id WHERE d.id=$1",
        )
        .bind(document)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?
        .ok_or_else(|| AppError::not_found("document", document))?;
        self.linked_projects(claims, &key).await
    }

    pub(super) async fn associate_project(
        &self,
        claims: &WikiClaims,
        key: &str,
        project: &NamespaceRef,
        attach: bool,
    ) -> Result<(), AppError> {
        if project.registry_instance_id.is_nil() || project.namespace_id.is_nil() {
            return Err(AppError::invalid_input("invalid_project_ref"));
        }
        let space = self
            .ensure_space_access(claims, key, WikiSpaceAccess::Admin)
            .await?;
        self.namespace_binding(space).await?;
        let actor = super::mapping::parse_uuid(&claims.user_id, "user")?;
        let catalog_visible = match self.project_catalog_actor(claims).await {
            Ok(_) => true,
            Err(AppError::Forbidden) => false,
            Err(error) => return Err(error),
        };
        let mut tx = self.pool.begin().await.map_err(database_error)?;
        sqlx::query("SELECT singleton FROM wiki_space_admission_state WHERE singleton FOR SHARE")
            .fetch_one(&mut *tx)
            .await
            .map_err(database_error)?;
        let tracker_owned: bool = sqlx::query_scalar(
            "SELECT tracker_owned FROM wiki_project_projection_state WHERE singleton FOR SHARE",
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(database_error)?;
        let admission_active: bool =
            sqlx::query_scalar("SELECT active FROM wiki_space_admission_state WHERE singleton")
                .fetch_one(&mut *tx)
                .await
                .map_err(database_error)?;
        if tracker_owned || admission_active {
            return Err(AppError::Unavailable(
                "project_links_owned_by_tracker_use_v2".into(),
            ));
        }
        // Same order as owner-v2: catalog barrier precedes the local space row.
        sqlx::query(
            "SELECT local_space_id FROM wiki_space_catalog WHERE local_space_id=$1 FOR SHARE",
        )
        .bind(space)
        .fetch_optional(&mut *tx)
        .await
        .map_err(database_error)?;
        // Serialize link edits with space archive. No document or project row is
        // moved when this association changes.
        let active = sqlx::query_scalar::<_, bool>(
            "SELECT archived_at IS NULL FROM spaces WHERE id=$1 FOR UPDATE",
        )
        .bind(space)
        .fetch_one(&mut *tx)
        .await
        .map_err(database_error)?;
        if !active {
            return Err(AppError::invalid_input(
                "archived space does not accept write commands",
            ));
        }
        if attach && sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM wiki_space_projects WHERE space_id=$1 AND registry_instance_id=$2 AND namespace_id=$3)",
        ).bind(space).bind(project.registry_instance_id).bind(project.namespace_id)
            .fetch_one(&mut *tx).await.map_err(database_error)? {
            tx.commit().await.map_err(database_error)?;
            return Ok(());
        }
        let changed = if attach {
            if !catalog_visible {
                return Err(AppError::Forbidden);
            }
            let row = sqlx::query(
                "SELECT w.presentation,(w.observed_at>now()-interval '5 minutes'
                 AND coalesce((SELECT available FROM wiki_workspace_catalog_sync),false)) AS available
                 FROM wiki_workspace_projects w WHERE w.registry_instance_id=$1
                 AND w.namespace_id=$2 AND w.present FOR SHARE OF w",
            ).bind(project.registry_instance_id).bind(project.namespace_id)
                .fetch_optional(&mut *tx).await.map_err(database_error)?
                .ok_or_else(|| AppError::not_found("project", project.namespace_id))?;
            if !row.get::<bool, _>("available") {
                return Err(AppError::Unavailable(
                    "tracker_project_catalog_unavailable".into(),
                ));
            }
            let item: ResourceContextSummary = serde_json::from_value(row.get("presentation"))
                .map_err(|_| AppError::Unavailable("invalid_tracker_projection".into()))?;
            let instance = std::env::var("WIKI_NAMESPACE__TRACKER_INSTANCE_ID")
                .ok()
                .and_then(|value| value.parse().ok())
                .ok_or_else(|| AppError::Unavailable("tracker_reader_not_configured".into()))?;
            let registry = std::env::var("WIKI_NAMESPACE__REGISTRY_INSTANCE_ID")
                .ok()
                .and_then(|value| value.parse().ok())
                .ok_or_else(|| AppError::Unavailable("namespace_owner_not_configured".into()))?;
            validate_project(&item, registry, instance)?;
            if item.binding.namespace != *project || item.binding.state != "active" {
                return Err(AppError::conflict("project_does_not_accept_links"));
            }
            sqlx::query(
                "INSERT INTO wiki_space_projects(space_id,registry_instance_id,namespace_id,created_by)
                 VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING",
            ).bind(space).bind(project.registry_instance_id).bind(project.namespace_id).bind(actor)
                .execute(&mut *tx).await.map_err(database_error)?.rows_affected()
        } else {
            // Removing a known association also works when its external project
            // was archived or is temporarily absent from the presentation cache.
            sqlx::query(
                "DELETE FROM wiki_space_projects WHERE space_id=$1 AND registry_instance_id=$2 AND namespace_id=$3",
            ).bind(space).bind(project.registry_instance_id).bind(project.namespace_id)
                .execute(&mut *tx).await.map_err(database_error)?.rows_affected()
        };
        if changed > 0 {
            sqlx::query("INSERT INTO audit_log(id,actor_id,action,entity_type,entity_id,diff,request_id) VALUES($1,$2,$3,'space',$4,$5,$6)")
                .bind(Uuid::now_v7()).bind(actor)
                .bind(if attach { "space.project_link" } else { "space.project_unlink" })
                .bind(space).bind(serde_json::json!({ "project": project }))
                .bind(super::audit::audit_request_id(claims.request_id.as_deref()))
                .execute(&mut *tx).await.map_err(database_error)?;
        }
        tx.commit().await.map_err(database_error)?;
        Ok(())
    }
}
