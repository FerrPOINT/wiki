//! Wiki is the canonical link owner. Reads never call Tracker back.
use super::{PostgresWikiBackend, ensure_document_accepts_writes_tx};
use shared::{
    AppError, managed_links::*, resource_context::NamespaceRef, wiki_contract::WikiClaims,
};
use sqlx::Row;
use uuid::Uuid;

pub(super) struct TaskReader {
    client: reqwest::Client,
    endpoint: reqwest::Url,
    token: String,
    instance: Uuid,
}

impl TaskReader {
    async fn list(
        &self,
        namespace: &NamespaceRef,
        offset: u32,
    ) -> Result<Vec<shared::resource_context::TaskCatalogItem>, AppError> {
        let endpoint = self
            .endpoint
            .join("api/v1/namespace-tasks")
            .map_err(|_| AppError::invalid_input("invalid_tracker_endpoint"))?;
        let mut response = self
            .client
            .get(endpoint)
            .query(&[
                (
                    "registry_instance_id",
                    namespace.registry_instance_id.to_string(),
                ),
                ("namespace_id", namespace.namespace_id.to_string()),
                ("offset", offset.to_string()),
            ])
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|_| AppError::Unavailable("tracker_reader_unavailable".into()))?;
        if !response.status().is_success() {
            return Err(AppError::Unavailable("tracker_reader_unavailable".into()));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| AppError::Unavailable("tracker_reader_unavailable".into()))?
        {
            if bytes.len() + chunk.len() > 65536 {
                return Err(AppError::Unavailable(
                    "tracker_reader_invalid_response".into(),
                ));
            }
            bytes.extend_from_slice(&chunk);
        }
        let items: Vec<shared::resource_context::TaskCatalogItem> = serde_json::from_slice(&bytes)
            .map_err(|_| AppError::Unavailable("tracker_reader_invalid_response".into()))?;
        if items.len() > 50
            || items.iter().any(|item| {
                item.namespace != *namespace
                    || item.task.tracker_instance_id != self.instance
                    || item.task.task_id.is_nil()
            })
        {
            return Err(AppError::Unavailable(
                "tracker_reader_invalid_response".into(),
            ));
        }
        Ok(items)
    }
    pub(super) fn from_deployment() -> Result<Option<Self>, AppError> {
        let Some(raw) = std::env::var("WIKI_NAMESPACE__TRACKER_URL").ok() else {
            return Ok(None);
        };
        let endpoint = reqwest::Url::parse(&raw)
            .map_err(|_| AppError::invalid_input("invalid_tracker_reader_endpoint"))?;
        if !matches!(endpoint.scheme(), "http" | "https")
            || endpoint.host_str().is_none()
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.path() != "/"
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
        {
            return Err(AppError::invalid_input("invalid_tracker_reader_endpoint"));
        }
        let instance = std::env::var("WIKI_NAMESPACE__TRACKER_INSTANCE_ID")
            .ok()
            .and_then(|v| v.parse().ok())
            .filter(|id: &Uuid| !id.is_nil())
            .ok_or_else(|| AppError::invalid_input("tracker_reader_instance_required"))?;
        let file = std::env::var("WIKI_NAMESPACE__TRACKER_TOKEN_FILE")
            .map_err(|_| AppError::invalid_input("tracker_reader_credential_required"))?;
        let token = std::fs::read_to_string(file)
            .map_err(|_| AppError::Unavailable("tracker_reader_credential_unavailable".into()))?
            .trim()
            .to_string();
        if token.is_empty() || token.contains(['\r', '\n']) {
            return Err(AppError::invalid_input("invalid_tracker_reader_credential"));
        }
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| AppError::Internal("tracker_reader_client_failed".into()))?;
        Ok(Some(Self {
            client,
            endpoint,
            token,
            instance,
        }))
    }
    async fn verify(&self, namespace: &NamespaceRef, task: &TaskRef) -> Result<String, AppError> {
        if task.tracker_instance_id != self.instance || task.task_id.is_nil() {
            return Err(AppError::invalid_input("unregistered_tracker_task_ref"));
        }
        let endpoint = self
            .endpoint
            .join(&format!("api/v1/namespace-tasks/{}", task.task_id))
            .map_err(|_| AppError::invalid_input("invalid_tracker_task_ref"))?;
        let mut response = self
            .client
            .get(endpoint)
            .query(&[
                (
                    "registry_instance_id",
                    namespace.registry_instance_id.to_string(),
                ),
                ("namespace_id", namespace.namespace_id.to_string()),
            ])
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|_| AppError::Unavailable("tracker_reader_unavailable".into()))?;
        if response.status().is_server_error()
            || response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
        {
            return Err(AppError::Unavailable("tracker_reader_unavailable".into()));
        }
        if !response.status().is_success() {
            return Err(AppError::invalid_input("task_ref_not_verified"));
        }
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| AppError::Unavailable("tracker_reader_unavailable".into()))?
        {
            if body.len() + chunk.len() > 65_536 {
                return Err(AppError::Unavailable(
                    "tracker_reader_invalid_response".into(),
                ));
            }
            body.extend_from_slice(&chunk);
        }
        let value: serde_json::Value = serde_json::from_slice(&body)
            .map_err(|_| AppError::Unavailable("tracker_reader_invalid_response".into()))?;
        if value["schema_version"] != 1
            || value["task_id"] != task.task_id.to_string()
            || value["tracker_instance_id"] != task.tracker_instance_id.to_string()
            || value["state"] != "active"
            || value["namespace"]
                != serde_json::to_value(namespace).map_err(|e| AppError::Internal(e.to_string()))?
        {
            return Err(AppError::invalid_input("task_ref_namespace_mismatch"));
        }
        value["task_key"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| AppError::Unavailable("tracker_reader_invalid_response".into()))
    }
}
impl PostgresWikiBackend {
    pub(super) async fn namespace_available_tasks(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        offset: u32,
    ) -> Result<Vec<shared::resource_context::TaskCatalogItem>, AppError> {
        let space = self
            .ensure_space_access(claims, space_key, app::wiki::WikiSpaceAccess::Edit)
            .await?;
        let binding = self
            .namespace_binding(space)
            .await?
            .ok_or_else(|| AppError::conflict("managed_space_binding_required"))?;
        self.task_reader
            .as_ref()
            .ok_or_else(|| AppError::Unavailable("tracker_reader_not_configured".into()))?
            .list(&binding.namespace, offset)
            .await
    }
    pub(super) async fn namespace_task_revision_links(
        &self,
        namespace: &NamespaceRef,
        task: &TaskRef,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let spaces: Vec<Uuid> = sqlx::query_scalar(
            "SELECT space_id FROM task_dossiers WHERE tracker_instance_id=$1 AND task_id=$2",
        )
        .bind(task.tracker_instance_id)
        .bind(task.task_id)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::database)?;
        for space in spaces {
            let binding = self
                .namespace_binding(space)
                .await?
                .ok_or_else(|| AppError::Unavailable("namespace_projection_missing".into()))?;
            if &binding.namespace != namespace {
                return Err(AppError::Forbidden);
            }
        }
        sqlx::query_scalar("SELECT jsonb_build_object('document_id',d.id,'revision_id',r.id,'title',r.title,'version',r.version,'space_key',s.key,'task_key',t.task_key,'namespace',jsonb_build_object('registry_instance_id',b.registry_instance_id,'namespace_id',b.namespace_id)) FROM managed_task_document_links l JOIN task_dossiers t ON t.id=l.dossier_id JOIN documents d ON d.id=l.document_id JOIN document_revisions r ON r.id=l.revision_id JOIN spaces s ON s.id=l.space_id JOIN wiki_namespace_bindings b ON b.resource_id=s.id WHERE t.tracker_instance_id=$1 AND t.task_id=$2 AND b.registry_instance_id=$3 AND b.namespace_id=$4 ORDER BY l.created_at DESC,l.id LIMIT 100")
            .bind(task.tracker_instance_id).bind(task.task_id).bind(namespace.registry_instance_id).bind(namespace.namespace_id).fetch_all(&self.pool).await.map_err(AppError::database)
    }
    pub(super) async fn link_task_revision(
        &self,
        claims: &WikiClaims,
        key: &str,
        input: &LinkTaskRevision,
    ) -> Result<serde_json::Value, AppError> {
        let space = self.space_id(key).await?;
        self.ensure_space_id_access(claims, space, app::wiki::WikiSpaceAccess::Edit)
            .await?;
        let binding = self
            .namespace_binding(space)
            .await?
            .ok_or_else(|| AppError::conflict("managed_space_binding_required"))?;
        if let Some(old)=sqlx::query("SELECT l.id,l.dossier_id FROM managed_task_document_links l JOIN task_dossiers t ON t.id=l.dossier_id WHERE l.space_id=$1 AND t.tracker_instance_id=$2 AND t.task_id=$3 AND l.document_id=$4 AND l.revision_id=$5").bind(space).bind(input.task.tracker_instance_id).bind(input.task.task_id).bind(input.document_id).bind(input.revision_id).fetch_optional(&self.pool).await.map_err(AppError::database)? {
            return Ok(serde_json::json!({"id":old.get::<Uuid,_>("id"),"dossier_id":old.get::<Uuid,_>("dossier_id"),"task":input.task,"document_id":input.document_id,"revision_id":input.revision_id}));
        }
        let reader = self
            .task_reader
            .as_ref()
            .ok_or_else(|| AppError::Unavailable("tracker_reader_not_configured".into()))?;
        let task_key = reader.verify(&binding.namespace, &input.task).await?;
        let mut tx = self.pool.begin().await.map_err(AppError::database)?;
        ensure_document_accepts_writes_tx(&mut tx, input.document_id).await?;
        let valid: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM document_revisions r JOIN documents d ON d.id=r.document_id WHERE r.id=$1 AND d.id=$2 AND d.space_id=$3)").bind(input.revision_id).bind(input.document_id).bind(space).fetch_one(&mut *tx).await.map_err(AppError::database)?;
        if !valid {
            return Err(AppError::invalid_input("revision_not_in_namespace_space"));
        }
        let dossier: Uuid = sqlx::query_scalar("INSERT INTO task_dossiers(id,space_id,task_key,tracker_instance_id,task_id) VALUES($1,$2,$3,$4,$5) ON CONFLICT(space_id,tracker_instance_id,task_id) WHERE task_id IS NOT NULL DO UPDATE SET task_key=EXCLUDED.task_key,updated_at=now() RETURNING id")
            .bind(Uuid::now_v7()).bind(space).bind(task_key).bind(input.task.tracker_instance_id).bind(input.task.task_id).fetch_one(&mut *tx).await.map_err(AppError::database)?;
        let actor: Uuid = claims.user_id.parse().map_err(|_| AppError::Unauthorized)?;
        let link: Uuid = sqlx::query_scalar("INSERT INTO managed_task_document_links(id,space_id,dossier_id,document_id,revision_id,created_by) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(dossier_id,document_id,revision_id) DO UPDATE SET dossier_id=EXCLUDED.dossier_id RETURNING id")
            .bind(Uuid::now_v7()).bind(space).bind(dossier).bind(input.document_id).bind(input.revision_id).bind(actor).fetch_one(&mut *tx).await.map_err(AppError::database)?;
        tx.commit().await.map_err(AppError::database)?;
        Ok(
            serde_json::json!({"id":link,"dossier_id":dossier,"task":input.task,"document_id":input.document_id,"revision_id":input.revision_id}),
        )
    }
    pub(super) async fn task_revision_links(
        &self,
        claims: &WikiClaims,
        task: &TaskRef,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let spaces: Vec<Uuid>=sqlx::query_scalar("SELECT DISTINCT space_id FROM task_dossiers WHERE tracker_instance_id=$1 AND task_id=$2").bind(task.tracker_instance_id).bind(task.task_id).fetch_all(&self.pool).await.map_err(AppError::database)?;
        for space in spaces {
            self.ensure_space_id_access(claims, space, app::wiki::WikiSpaceAccess::View)
                .await?;
            self.namespace_binding(space)
                .await?
                .ok_or_else(|| AppError::Unavailable("namespace_projection_missing".into()))?;
        }
        let rows = sqlx::query("SELECT l.space_id,jsonb_build_object('id',l.id,'document_id',d.id,'revision_id',r.id,'title',r.title,'version',r.version,'space_key',s.key,'task_key',t.task_key,'namespace',jsonb_build_object('registry_instance_id',b.registry_instance_id,'namespace_id',b.namespace_id)) AS document FROM managed_task_document_links l JOIN task_dossiers t ON t.id=l.dossier_id JOIN documents d ON d.id=l.document_id JOIN document_revisions r ON r.id=l.revision_id JOIN spaces s ON s.id=l.space_id JOIN wiki_namespace_bindings b ON b.resource_id=s.id WHERE t.tracker_instance_id=$1 AND t.task_id=$2 ORDER BY l.created_at DESC,l.id LIMIT 100")
            .bind(task.tracker_instance_id).bind(task.task_id).fetch_all(&self.pool).await.map_err(AppError::database)?;
        let mut result = Vec::new();
        for row in rows {
            self.ensure_space_id_access(
                claims,
                row.get("space_id"),
                app::wiki::WikiSpaceAccess::View,
            )
            .await?;
            result.push(row.get("document"));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn reader(status: u16, body: String, instance: Uuid) -> TaskReader {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            loop {
                let mut chunk = [0_u8; 1024];
                let count = stream.read(&mut chunk).await.unwrap();
                assert_ne!(count, 0);
                request.extend_from_slice(&chunk[..count]);
                if request.windows(4).any(|part| part == b"\r\n\r\n") {
                    break;
                }
                assert!(request.len() < 8192);
            }
            let request = String::from_utf8(request).unwrap();
            assert!(
                request
                    .to_lowercase()
                    .contains("authorization: bearer qa-reader-only")
            );
            let response = format!(
                "HTTP/1.1 {status} QA\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        TaskReader {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(2))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
            endpoint: reqwest::Url::parse(&format!("http://{address}/")).unwrap(),
            token: "qa-reader-only".into(),
            instance,
        }
    }

    #[tokio::test]
    async fn tracker_temporary_failures_are_unavailable() {
        let namespace = NamespaceRef {
            registry_instance_id: Uuid::now_v7(),
            namespace_id: Uuid::now_v7(),
        };
        let task = TaskRef {
            tracker_instance_id: Uuid::now_v7(),
            task_id: Uuid::now_v7(),
        };
        for status in [500, 503, 429] {
            let reader = reader(status, String::new(), task.tracker_instance_id).await;
            let result = reader.verify(&namespace, &task).await;
            assert!(
                matches!(result, Err(AppError::Unavailable(_))),
                "upstream {status} must remain a temporary dependency failure: {result:?}"
            );
        }
    }

    #[tokio::test]
    async fn tracker_missing_task_remains_invalid_input() {
        let namespace = NamespaceRef {
            registry_instance_id: Uuid::now_v7(),
            namespace_id: Uuid::now_v7(),
        };
        let task = TaskRef {
            tracker_instance_id: Uuid::now_v7(),
            task_id: Uuid::now_v7(),
        };
        let reader = reader(404, String::new(), task.tracker_instance_id).await;
        assert!(matches!(
            reader.verify(&namespace, &task).await,
            Err(AppError::InvalidInput(_))
        ));
    }
}
