//! Product mapping and bounded server admission. No UI input is trusted as proof.
use super::{PostgresWikiBackend, central_auth::BRIDGE};
use pdlc_auth_core::service_bridge::{
    SpaceAdmissionClient, VerifiedBridgeError, VerifiedCentralToken,
};
use pdlc_shared::space_access::{AdmissionValidity, CentralIdentity, SpaceRef};
use shared::{
    AppError, WikiClaims,
    wiki_space_access::{WikiSpaceAdmission, WikiSpaceMapping, WikiSpaceSubject},
};
use std::sync::Arc;
use uuid::Uuid;

static CLIENT: tokio::sync::OnceCell<SpaceAdmissionClient> = tokio::sync::OnceCell::const_new();
async fn client() -> Result<&'static SpaceAdmissionClient, AppError> {
    CLIENT
        .get_or_try_init(|| async {
            let url = std::env::var("WIKI_SPACES__ADMISSION_URL")
                .map_err(|_| AppError::Unavailable("space_admission_not_configured".into()))?;
            let token = std::env::var("WIKI_SPACES__ADMISSION_TOKEN")
                .map_err(|_| AppError::Unavailable("space_admission_not_configured".into()))?;
            SpaceAdmissionClient::new(&url, token)
                .map_err(|_| AppError::Unavailable("space_admission_configuration_invalid".into()))
        })
        .await
}
pub(super) fn auth_error(error: VerifiedBridgeError) -> AppError {
    match error {
        VerifiedBridgeError::Unavailable | VerifiedBridgeError::NotConfigured => {
            AppError::Unavailable("central_confirmation_unavailable".into())
        }
        _ => AppError::Unauthorized,
    }
}
impl PostgresWikiBackend {
    pub(super) async fn admission_active(&self) -> Result<bool, AppError> {
        sqlx::query_scalar("SELECT active FROM wiki_space_admission_state WHERE singleton")
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::database)
    }
    pub(super) async fn read_admission(
        &self,
        claims: Option<&WikiClaims>,
    ) -> Result<Option<Arc<WikiSpaceAdmission>>, AppError> {
        let active = self.admission_active().await?;
        if let Some(claims) = claims {
            if let Some(admission) = claims.admission.as_ref() {
                let user = Uuid::parse_str(&claims.user_id).map_err(|_| AppError::Unauthorized)?;
                if !admission.matches_user(user) {
                    return Err(AppError::Forbidden);
                }
                return Ok(Some(admission.clone()));
            }
        }
        if active {
            Err(AppError::Forbidden)
        } else {
            Ok(None)
        }
    }
    pub(super) async fn draft_read_admitted_spaces(
        &self,
        claims: &WikiClaims,
        admission: Option<&Arc<WikiSpaceAdmission>>,
    ) -> Result<Option<Vec<Uuid>>, AppError> {
        let Some(scope) = admission else {
            return Ok(None);
        };
        let allowed = scope.draft_readable_ids()?;
        let user = Uuid::parse_str(&claims.user_id).map_err(|_| AppError::Unauthorized)?;
        let ids=sqlx::query_scalar("SELECT s.id FROM spaces s JOIN users u ON u.id=$1 AND u.is_active LEFT JOIN space_members sm ON sm.space_id=s.id AND sm.user_id=u.id WHERE s.id=ANY($2::uuid[]) AND s.archived_at IS NULL AND EXISTS(SELECT 1 FROM wiki_space_catalog c WHERE c.local_space_id=s.id AND c.admin_revision IS NOT NULL AND c.state='active') AND (u.global_role='admin' OR sm.role IN ('admin','editor')) ORDER BY s.id")
            .bind(user).bind(allowed).fetch_all(&self.pool).await.map_err(AppError::database)?;
        scope.require_current()?;
        Ok(Some(ids))
    }
    pub(super) async fn may_read_document_drafts(
        &self,
        claims: &WikiClaims,
        space_id: Uuid,
    ) -> Result<bool, AppError> {
        let admission = self.read_admission(Some(claims)).await?;
        if let Some(scope) = admission.as_ref() {
            if !scope.allows_draft_read(space_id) {
                return Ok(false);
            }
        }
        let user = Uuid::parse_str(&claims.user_id).map_err(|_| AppError::Unauthorized)?;
        let allowed:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users u LEFT JOIN space_members sm ON sm.user_id=u.id AND sm.space_id=$2 WHERE u.id=$1 AND u.is_active AND (u.global_role='admin' OR sm.role IN ('admin','editor')))")
            .bind(user).bind(space_id).fetch_one(&self.pool).await.map_err(AppError::database)?;
        if let Some(scope) = admission.as_ref() {
            scope.require_current()?;
        }
        Ok(allowed)
    }
    pub(super) async fn confirmed_admission(
        &self,
        actor: &VerifiedCentralToken,
        user_id: Uuid,
        global_role: String,
    ) -> Result<Arc<WikiSpaceAdmission>, AppError> {
        let rows:Vec<(Uuid,Uuid,Uuid,Option<String>)>=sqlx::query_as("SELECT c.local_space_id,c.registry_instance_id,c.space_id,sm.role FROM wiki_space_catalog c LEFT JOIN space_members sm ON sm.space_id=c.local_space_id AND sm.user_id=$1 ORDER BY c.registry_instance_id,c.space_id LIMIT 10001")
            .bind(user_id).fetch_all(&self.pool).await.map_err(AppError::database)?;
        if rows.len() > 10000 {
            return Err(AppError::Unavailable(
                "space_admission_catalog_too_large".into(),
            ));
        }
        let mappings: Vec<_> = rows
            .into_iter()
            .map(
                |(local_space_id, registry_instance_id, space_id, role)| WikiSpaceMapping {
                    local_space_id,
                    space: SpaceRef {
                        registry_instance_id,
                        space_id,
                    },
                    may_edit: global_role == "admin"
                        || matches!(role.as_deref(), Some("admin" | "editor")),
                    may_admin: global_role == "admin" || role.as_deref() == Some("admin"),
                },
            )
            .collect();
        let spaces: Vec<_> = mappings
            .iter()
            .map(|mapping| mapping.space.clone())
            .collect();
        let admission = client()
            .await?
            .confirm_many(&BRIDGE, actor, &spaces)
            .await
            .map_err(|_| AppError::Unavailable("space_admission_unavailable".into()))?;
        if !actor.lifetime.is_current() {
            return Err(AppError::Unauthorized);
        }
        let token_expiry = actor.lifetime.local_valid_until().min(
            actor
                .lifetime
                .source_observed_at
                .checked_add(AdmissionValidity::MAX_AGE_SECONDS)
                .ok_or(AppError::Unauthorized)?,
        );
        let subject = WikiSpaceSubject {
            user_id,
            identity: CentralIdentity {
                issuer: actor.issuer.clone(),
                subject: actor.context.user_id.clone(),
            },
            token_expiry,
            global_role,
            may_read: actor.context.allows_service("wiki", "GET"),
            may_write: actor.context.allows_service("wiki", "PUT"),
        };
        Ok(Arc::new(WikiSpaceAdmission::confirmed(
            subject, admission, mappings,
        )?))
    }
}
