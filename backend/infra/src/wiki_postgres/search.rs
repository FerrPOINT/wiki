use super::{
    PostgresWikiBackend,
    mapping::search_result_from_row,
    queries::{SEARCH_DOCUMENTS_SQL, SEARCH_EVIDENCE_SQL},
};
use app::wiki::{WikiSpaceAccess as SpaceAccess, build_wiki_search_criteria};
use shared::wiki_contract::*;

impl PostgresWikiBackend {
    pub(super) async fn search(
        &self,
        claims: &WikiClaims,
        query: SearchQuery,
    ) -> Result<SearchResponse, shared::AppError> {
        let criteria = build_wiki_search_criteria(
            query.q.as_deref(),
            query.space.as_deref(),
            query.task_key.as_deref(),
            query.phase_key.as_deref(),
            query.document_type.as_deref(),
            query.include_archived,
            query.limit,
        )?;
        if let Some(space_key) = criteria.space_key.as_deref() {
            self.ensure_space_access(claims, space_key, SpaceAccess::View)
                .await?;
        }
        let access_user_id = self.restricted_user_id(claims).await?;

        let document_rows = sqlx::query(SEARCH_DOCUMENTS_SQL)
            .bind(&criteria.needle)
            .bind(criteria.space_key.as_deref())
            .bind(criteria.task_key.as_deref())
            .bind(criteria.phase_key.as_deref())
            .bind(criteria.document_type)
            .bind(criteria.include_archived)
            .bind(access_user_id)
            .bind(criteria.limit)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        let evidence_rows = sqlx::query(SEARCH_EVIDENCE_SQL)
            .bind(&criteria.evidence_like_pattern)
            .bind(criteria.space_key.as_deref())
            .bind(criteria.task_key.as_deref())
            .bind(criteria.phase_key.as_deref())
            .bind(access_user_id)
            .bind(criteria.limit)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;

        let mut results = document_rows
            .iter()
            .map(search_result_from_row)
            .chain(evidence_rows.iter().map(search_result_from_row))
            .collect::<Vec<_>>();
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        results.truncate(criteria.limit as usize);
        Ok(SearchResponse { results })
    }
}
