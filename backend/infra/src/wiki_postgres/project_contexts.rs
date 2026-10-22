//! Fresh owner context reads; the local M:N projection is never the collection source.
use super::PostgresWikiBackend;
use super::database_error;
use app::wiki::WikiSpaceAccess;
use pdlc_shared::{project_spaces::*, space_access::*};
use shared::{AppError, wiki_contract::*};
use sqlx::Row;
impl PostgresWikiBackend {
    pub(super) async fn canonical_space_projects(
        &self,
        claims: &WikiClaims,
        key: &str,
        limit: u32,
        offset: u32,
    ) -> Result<SpaceProjectsV2, AppError> {
        if !(1..=100).contains(&limit) {
            return Err(AppError::invalid_input("invalid_project_context_page"));
        }
        let scope = claims.admission.as_ref().ok_or_else(|| {
            AppError::Unavailable("space_admission_required_use_compatible_cohort".into())
        })?;
        let local = self
            .ensure_space_access(claims, key, WikiSpaceAccess::View)
            .await?;
        let public = scope.readable_space_ref(local)?;
        let reader = self.task_reader.as_ref().ok_or_else(|| {
            AppError::Unavailable("tracker_project_context_not_configured".into())
        })?;
        let request = ProjectSpaceContextQuery {
            schema_version: 1,
            tracker_instance_id: reader.instance_id(),
            actor: scope.read_delegation()?,
            policy_revision: scope.policy_revision()?,
            selector: ProjectContextSelector::Space {
                space: public.clone(),
                limit,
                offset,
            },
        };
        let row=sqlx::query("SELECT s.key,s.name,c.registry_instance_id,c.space_id,c.admin_revision,c.state FROM spaces s JOIN wiki_space_catalog c ON c.local_space_id=s.id WHERE s.id=$1").bind(local).fetch_optional(&self.pool).await.map_err(database_error)?.ok_or_else(||AppError::Unavailable("shared_space_mapping_unconfirmed".into()))?;
        let mapped = SpaceRef {
            registry_instance_id: row
                .try_get("registry_instance_id")
                .map_err(database_error)?,
            space_id: row.try_get("space_id").map_err(database_error)?,
        };
        let revision: Option<i64> = row.try_get("admin_revision").map_err(database_error)?;
        if mapped != public || revision.is_none_or(|r| r < 1) {
            return Err(AppError::Unavailable(
                "shared_space_mapping_unconfirmed".into(),
            ));
        }
        let context = reader.canonical_project_contexts(&request).await?;
        scope.require_current()?;
        if !scope.allows(local, false) {
            return Err(AppError::Forbidden);
        }
        Ok(SpaceProjectsV2 {
            schema_version: 2,
            space: public,
            wiki: WorkspaceSpace {
                id: local,
                key: row.try_get("key").map_err(database_error)?,
                name: row.try_get("name").map_err(database_error)?,
                status: row.try_get("state").map_err(database_error)?,
                resource_status: ProjectResourceStatus::Connected,
            },
            revision: revision.unwrap(),
            policy_revision: context.policy_revision,
            validity: context.actor.validity,
            total: context.total,
            has_more: context.has_more,
            projects: context.projects,
        })
    }
}
