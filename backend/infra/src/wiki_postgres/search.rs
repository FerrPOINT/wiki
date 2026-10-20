use super::{
    PostgresWikiBackend,
    mapping::search_result_from_row,
    queries::{SEARCH_DOCUMENTS_SQL, SEARCH_EVIDENCE_SQL},
};
use app::wiki::{
    WikiSearchCriteria, WikiSearchRepository, WikiSearchRepositoryFuture, WikiSearchUseCase,
    WikiSpaceAccess as SpaceAccess, build_wiki_search_criteria_from_query,
};
use shared::wiki_contract::*;
use uuid::Uuid;

struct PostgresWikiSearchRepository<'a> {
    pool: &'a sqlx::PgPool,
    admission: Option<std::sync::Arc<shared::wiki_space_access::WikiSpaceAdmission>>,
    editable_spaces: Option<Vec<Uuid>>,
}

impl WikiSearchRepository for PostgresWikiSearchRepository<'_> {
    fn search_documents<'a>(
        &'a self,
        criteria: &'a WikiSearchCriteria,
        restricted_user_id: Option<Uuid>,
    ) -> WikiSearchRepositoryFuture<'a> {
        Box::pin(async move {
            let rows = sqlx::query(SEARCH_DOCUMENTS_SQL)
                .bind(&criteria.needle)
                .bind(criteria.space_key.as_deref())
                .bind(criteria.task_key.as_deref())
                .bind(criteria.phase_key.as_deref())
                .bind(criteria.document_type)
                .bind(criteria.include_archived)
                .bind(restricted_user_id)
                .bind(
                    criteria
                        .cursor
                        .as_ref()
                        .map(|cursor| cursor.updated_at.to_owned()),
                )
                .bind(criteria.cursor.as_ref().map(|cursor| cursor.id))
                .bind(
                    criteria
                        .cursor
                        .as_ref()
                        .map(|cursor| cursor.result_type.as_str()),
                )
                .bind(criteria.limit + 1)
                .bind(
                    self.admission
                        .as_ref()
                        .map(|scope| scope.readable_ids())
                        .transpose()?,
                )
                .bind(&self.editable_spaces)
                .fetch_all(self.pool)
                .await
                .map_err(shared::AppError::database)?;
            if let Some(scope) = self.admission.as_ref() {
                scope.require_current()?;
            }
            Ok(rows.iter().map(search_result_from_row).collect())
        })
    }

    fn search_evidence<'a>(
        &'a self,
        criteria: &'a WikiSearchCriteria,
        restricted_user_id: Option<Uuid>,
    ) -> WikiSearchRepositoryFuture<'a> {
        Box::pin(async move {
            let rows = sqlx::query(SEARCH_EVIDENCE_SQL)
                .bind(&criteria.evidence_like_pattern)
                .bind(criteria.space_key.as_deref())
                .bind(criteria.task_key.as_deref())
                .bind(criteria.phase_key.as_deref())
                .bind(restricted_user_id)
                .bind(
                    criteria
                        .cursor
                        .as_ref()
                        .map(|cursor| cursor.updated_at.to_owned()),
                )
                .bind(criteria.cursor.as_ref().map(|cursor| cursor.id))
                .bind(
                    criteria
                        .cursor
                        .as_ref()
                        .map(|cursor| cursor.result_type.as_str()),
                )
                .bind(criteria.limit + 1)
                .bind(
                    self.admission
                        .as_ref()
                        .map(|scope| scope.readable_ids())
                        .transpose()?,
                )
                .fetch_all(self.pool)
                .await
                .map_err(shared::AppError::database)?;
            if let Some(scope) = self.admission.as_ref() {
                scope.require_current()?;
            }
            Ok(rows.iter().map(search_result_from_row).collect())
        })
    }
}

impl PostgresWikiBackend {
    pub(super) async fn search(
        &self,
        claims: &WikiClaims,
        query: SearchQuery,
    ) -> Result<SearchResponse, shared::AppError> {
        let criteria = build_wiki_search_criteria_from_query(&query)?;
        if let Some(space_key) = criteria.space_key.as_deref() {
            self.ensure_space_access(claims, space_key, SpaceAccess::View)
                .await?;
        }
        let access_user_id = self.restricted_user_id(claims).await?;
        let admission = self.read_admission(Some(claims)).await?;
        let editable_spaces = self
            .draft_read_admitted_spaces(claims, admission.as_ref())
            .await?;
        let repository = PostgresWikiSearchRepository {
            pool: &self.pool,
            admission,
            editable_spaces,
        };
        WikiSearchUseCase::new(&repository)
            .execute(criteria, access_user_id)
            .await
    }
}
