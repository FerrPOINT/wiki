use super::{
    PostgresWikiBackend,
    mapping::{document_summary_from_row, evidence_response_from_row, parse_uuid},
    queries::EVIDENCE_TARGET_SQL,
};
use app::wiki::{
    WikiSpaceAccess as SpaceAccess, normalize_phase_key, normalize_space_key, normalize_task_key,
};
use shared::wiki_contract::*;
use sqlx::{Postgres, Row};
use uuid::Uuid;

impl PostgresWikiBackend {
    pub(super) async fn list_tasks(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<TaskPageListResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::View)
            .await?;
        let rows = sqlx::query(
            r#"
            SELECT task_key
            FROM task_dossiers
            WHERE space_id = $1
            ORDER BY task_key
            "#,
        )
        .bind(space_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let mut tasks = Vec::with_capacity(rows.len());
        for row in rows {
            let task_key: String = row.get("task_key");
            tasks.push(self.task_page(&key, &task_key).await?);
        }
        Ok(TaskPageListResponse { tasks })
    }

    pub(super) async fn get_task(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<TaskPageResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        self.ensure_space_access(claims, &key, SpaceAccess::View)
            .await?;
        let task_key = normalize_task_key(task_key)?;
        self.task_page(&key, &task_key).await
    }

    pub(super) async fn link_task_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
        body: LinkDocumentRequest,
    ) -> Result<TaskPageResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let task_key = normalize_task_key(task_key)?;
        let document_id = self.resolve_document_id(&body.document_id).await?;
        if self.document_space_id(document_id).await? != space_id {
            return Err(shared::AppError::invalid_input(
                "document belongs to another space",
            ));
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let task_id = self
            .upsert_task_dossier_tx(&mut tx, space_id, &task_key)
            .await?;
        sqlx::query(
            r#"
            INSERT INTO document_task_links (space_id, document_id, task_dossier_id, created_by, created_at)
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(space_id)
        .bind(document_id)
        .bind(task_id)
        .bind(actor_id)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        self.insert_audit(
            &mut tx,
            Some(actor_id),
            "task.link_document",
            "task",
            task_id,
        )
        .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        self.task_page(&key, &task_key).await
    }

    pub(super) async fn list_task_documents(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<DocumentListResponse, shared::AppError> {
        let task = self.get_task(claims, space_key, task_key).await?;
        let mut documents = Vec::with_capacity(task.documents.len());
        for summary in task.documents {
            documents.push(
                self.document_response(parse_uuid(&summary.id, "document")?)
                    .await?,
            );
        }
        Ok(DocumentListResponse { documents })
    }

    pub(super) async fn list_task_evidence(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        Ok(EvidenceListResponse {
            evidence: self.get_task(claims, space_key, task_key).await?.evidence,
        })
    }

    pub(super) async fn list_phases(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<PhasePageListResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::View)
            .await?;
        let rows = sqlx::query(
            r#"
            SELECT phase_key
            FROM phase_dossiers
            WHERE space_id = $1
            ORDER BY phase_key
            "#,
        )
        .bind(space_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let mut phases = Vec::with_capacity(rows.len());
        for row in rows {
            let phase_key: String = row.get("phase_key");
            phases.push(self.phase_page(&key, &phase_key).await?);
        }
        Ok(PhasePageListResponse { phases })
    }

    pub(super) async fn get_phase(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<PhasePageResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        self.ensure_space_access(claims, &key, SpaceAccess::View)
            .await?;
        let phase_key = normalize_phase_key(phase_key)?;
        self.phase_page(&key, &phase_key).await
    }

    pub(super) async fn link_phase_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
        body: LinkDocumentRequest,
    ) -> Result<PhasePageResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let phase_key = normalize_phase_key(phase_key)?;
        let document_id = self.resolve_document_id(&body.document_id).await?;
        if self.document_space_id(document_id).await? != space_id {
            return Err(shared::AppError::invalid_input(
                "document belongs to another space",
            ));
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let phase_id = self
            .upsert_phase_dossier_tx(&mut tx, space_id, &phase_key)
            .await?;
        sqlx::query(
            r#"
            INSERT INTO document_phase_links (space_id, document_id, phase_dossier_id, created_by, created_at)
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(space_id)
        .bind(document_id)
        .bind(phase_id)
        .bind(actor_id)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        self.insert_audit(
            &mut tx,
            Some(actor_id),
            "phase.link_document",
            "phase",
            phase_id,
        )
        .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        self.phase_page(&key, &phase_key).await
    }

    pub(super) async fn list_phase_documents(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<DocumentListResponse, shared::AppError> {
        let phase = self.get_phase(claims, space_key, phase_key).await?;
        let mut documents = Vec::with_capacity(phase.documents.len());
        for summary in phase.documents {
            documents.push(
                self.document_response(parse_uuid(&summary.id, "document")?)
                    .await?,
            );
        }
        Ok(DocumentListResponse { documents })
    }

    pub(super) async fn list_phase_evidence(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        Ok(EvidenceListResponse {
            evidence: self.get_phase(claims, space_key, phase_key).await?.evidence,
        })
    }

    async fn task_page(
        &self,
        space_key: &str,
        task_key: &str,
    ) -> Result<TaskPageResponse, shared::AppError> {
        let space_id = self.space_id(space_key).await?;
        let task_row = sqlx::query(
            "SELECT id, title_snapshot FROM task_dossiers WHERE space_id = $1 AND task_key = $2",
        )
        .bind(space_id)
        .bind(task_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let Some(task_row) = task_row else {
            return Ok(TaskPageResponse {
                space_key: space_key.to_string(),
                task_key: task_key.to_string(),
                title: None,
                document_count: 0,
                evidence_count: 0,
                documents: Vec::new(),
                evidence: Vec::new(),
            });
        };
        let task_id: Uuid = task_row.get("id");
        let document_rows = sqlx::query(
            r#"
            SELECT d.id, d.slug, d.title, d.document_type, d.status, d.updated_at
            FROM document_task_links dtl
            JOIN documents d ON d.id = dtl.document_id
            WHERE dtl.task_dossier_id = $1 AND d.archived_at IS NULL
            ORDER BY d.updated_at DESC
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let documents: Vec<_> = document_rows
            .iter()
            .map(document_summary_from_row)
            .collect();
        let evidence = self.evidence_for_target(Some(task_id), None).await?;
        let title_snapshot: Option<String> = task_row.get("title_snapshot");
        let title =
            title_snapshot.or_else(|| documents.first().map(|document| document.title.clone()));
        Ok(TaskPageResponse {
            space_key: space_key.to_string(),
            task_key: task_key.to_string(),
            title,
            document_count: documents.len(),
            evidence_count: evidence.len(),
            documents,
            evidence,
        })
    }

    async fn phase_page(
        &self,
        space_key: &str,
        phase_key: &str,
    ) -> Result<PhasePageResponse, shared::AppError> {
        let space_id = self.space_id(space_key).await?;
        let phase_row = sqlx::query(
            "SELECT id, phase_name FROM phase_dossiers WHERE space_id = $1 AND phase_key = $2",
        )
        .bind(space_id)
        .bind(phase_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let Some(phase_row) = phase_row else {
            return Ok(PhasePageResponse {
                space_key: space_key.to_string(),
                phase_key: phase_key.to_string(),
                title: Some(phase_key.to_string()),
                document_count: 0,
                evidence_count: 0,
                documents: Vec::new(),
                evidence: Vec::new(),
            });
        };
        let phase_id: Uuid = phase_row.get("id");
        let document_rows = sqlx::query(
            r#"
            SELECT d.id, d.slug, d.title, d.document_type, d.status, d.updated_at
            FROM document_phase_links dpl
            JOIN documents d ON d.id = dpl.document_id
            WHERE dpl.phase_dossier_id = $1 AND d.archived_at IS NULL
            ORDER BY d.updated_at DESC
            "#,
        )
        .bind(phase_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let documents: Vec<_> = document_rows
            .iter()
            .map(document_summary_from_row)
            .collect();
        let evidence = self.evidence_for_target(None, Some(phase_id)).await?;
        let phase_name: Option<String> = phase_row.get("phase_name");
        Ok(PhasePageResponse {
            space_key: space_key.to_string(),
            phase_key: phase_key.to_string(),
            title: phase_name.or_else(|| Some(phase_key.to_string())),
            document_count: documents.len(),
            evidence_count: evidence.len(),
            documents,
            evidence,
        })
    }

    async fn evidence_for_target(
        &self,
        task_dossier_id: Option<Uuid>,
        phase_dossier_id: Option<Uuid>,
    ) -> Result<Vec<EvidenceResponse>, shared::AppError> {
        let rows = sqlx::query(EVIDENCE_TARGET_SQL)
            .bind(task_dossier_id)
            .bind(phase_dossier_id)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        Ok(rows.iter().map(evidence_response_from_row).collect())
    }

    pub(super) async fn upsert_task_dossier_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        space_id: Uuid,
        task_key: &str,
    ) -> Result<Uuid, shared::AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO task_dossiers (id, space_id, task_key, created_at, updated_at)
            VALUES ($1, $2, $3, now(), now())
            ON CONFLICT (space_id, task_key)
            DO UPDATE SET updated_at = now()
            RETURNING id
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(space_id)
        .bind(task_key)
        .fetch_one(&mut **tx)
        .await
        .map_err(shared::AppError::database)?;
        Ok(row.get("id"))
    }

    pub(super) async fn upsert_phase_dossier_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        space_id: Uuid,
        phase_key: &str,
    ) -> Result<Uuid, shared::AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO phase_dossiers (id, space_id, phase_key, created_at, updated_at)
            VALUES ($1, $2, $3, now(), now())
            ON CONFLICT (space_id, phase_key)
            DO UPDATE SET updated_at = now()
            RETURNING id
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(space_id)
        .bind(phase_key)
        .fetch_one(&mut **tx)
        .await
        .map_err(shared::AppError::database)?;
        Ok(row.get("id"))
    }
}
