use super::{PostgresWikiBackend, mapping::template_response_from_row};
use app::wiki::{
    WikiCreateTemplateCommand, WikiTemplateRepository, WikiTemplateRepositoryFuture,
    WikiTemplateUseCase,
};
use shared::wiki_contract::*;
use uuid::Uuid;

struct PostgresWikiTemplateRepository<'a> {
    backend: &'a PostgresWikiBackend,
    request_id: Option<&'a str>,
}

impl WikiTemplateRepository for PostgresWikiTemplateRepository<'_> {
    fn list_active_templates<'a>(
        &'a self,
    ) -> WikiTemplateRepositoryFuture<'a, Vec<TemplateResponse>> {
        Box::pin(async move {
            let rows = sqlx::query(
                r#"
                SELECT id, name, document_type, content_markdown
                FROM document_templates
                WHERE is_active = true
                ORDER BY lower(name)
                "#,
            )
            .fetch_all(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?;
            Ok(rows.iter().map(template_response_from_row).collect())
        })
    }

    fn create_template<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiCreateTemplateCommand,
    ) -> WikiTemplateRepositoryFuture<'a, TemplateResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;
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
            .bind(&command.name)
            .bind(&command.document_type)
            .bind(&command.body_markdown)
            .fetch_one(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;
            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "template.create",
                    "template",
                    id,
                    self.request_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;
            Ok(template_response_from_row(&row))
        })
    }
}

impl PostgresWikiBackend {
    pub(super) async fn list_templates(&self) -> Result<TemplateListResponse, shared::AppError> {
        let repository = PostgresWikiTemplateRepository {
            backend: self,
            request_id: None,
        };
        WikiTemplateUseCase::new(&repository).list().await
    }

    pub(super) async fn create_template(
        &self,
        claims: &WikiClaims,
        body: CreateTemplateRequest,
    ) -> Result<TemplateResponse, shared::AppError> {
        let actor_id = self.ensure_admin(claims).await?;
        let repository = PostgresWikiTemplateRepository {
            backend: self,
            request_id: claims.request_id.as_deref(),
        };
        WikiTemplateUseCase::new(&repository)
            .create(actor_id, body)
            .await
    }
}
