use super::{PostgresWikiBackend, mapping::template_response_from_row};
use app::wiki::{normalize_document_type, normalize_required};
use shared::wiki_contract::*;
use uuid::Uuid;

impl PostgresWikiBackend {
    pub(super) async fn list_templates(&self) -> Result<TemplateListResponse, shared::AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, document_type, content_markdown
            FROM document_templates
            WHERE is_active = true
            ORDER BY lower(name)
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(TemplateListResponse {
            templates: rows.iter().map(template_response_from_row).collect(),
        })
    }

    pub(super) async fn create_template(
        &self,
        claims: &WikiClaims,
        body: CreateTemplateRequest,
    ) -> Result<TemplateResponse, shared::AppError> {
        let actor_id = self.ensure_admin(claims).await?;
        let name = normalize_required(&body.name, "template name")?;
        let document_type = normalize_document_type(&body.document_type, false)?;
        let body_markdown = normalize_required(&body.body_markdown, "template body_markdown")?;
        let id = Uuid::now_v7();
        let row = sqlx::query(
            r#"
            INSERT INTO document_templates (
                id, name, document_type, content_markdown, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, true, now(), now())
            RETURNING id, name, document_type, content_markdown
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(document_type)
        .bind(body_markdown)
        .fetch_one(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        self.audit(Some(actor_id), "template.create", "template", id)
            .await?;
        Ok(template_response_from_row(&row))
    }
}
