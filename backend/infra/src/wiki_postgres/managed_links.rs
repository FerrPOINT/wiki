//! Wiki is the canonical link owner. Reads never call Tracker back.
use super::{PostgresWikiBackend, ensure_document_accepts_writes_tx};
use shared::{
    AppError, managed_links::*, resource_context::NamespaceRef, wiki_contract::WikiClaims,
};
use sqlx::Row;
use uuid::Uuid;

fn tracker_reader_client(
    builder: reqwest::ClientBuilder,
    timeout: std::time::Duration,
) -> Result<reqwest::Client, reqwest::Error> {
    builder
        .no_proxy()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
}

pub(super) struct TaskReader {
    client: reqwest::Client,
    endpoint: reqwest::Url,
    token: String,
    instance: Uuid,
}

impl TaskReader {
    pub(super) fn instance_id(&self) -> Uuid {
        self.instance
    }
    /// Reuse the deployment reader transport/credential; user tokens and roles are not relayed.
    pub(super) async fn canonical_project_contexts(
        &self,
        request: &pdlc_shared::project_spaces::ProjectSpaceContextQuery,
    ) -> Result<pdlc_shared::project_spaces::ProjectSpaceContextBatch, AppError> {
        if !request.valid_for(&request.actor.identity.issuer, self.instance) {
            return Err(AppError::invalid_input("invalid_project_context_query"));
        }
        let url = self
            .endpoint
            .join("api/v2/project-contexts/query")
            .map_err(|_| AppError::Unavailable("invalid_tracker_context_endpoint".into()))?;
        let mut response = self
            .client
            .post(url)
            .bearer_auth(&self.token)
            .json(request)
            .send()
            .await
            .map_err(|_| AppError::Unavailable("tracker_project_context_unavailable".into()))?;
        if response.status() == reqwest::StatusCode::CONFLICT {
            return Err(AppError::conflict("space_policy_revision_changed"));
        }
        if !response.status().is_success() {
            return Err(AppError::Unavailable(
                "tracker_project_context_unavailable".into(),
            ));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| AppError::Unavailable("tracker_project_context_read_failed".into()))?
        {
            if bytes.len().saturating_add(chunk.len()) > 1024 * 1024 {
                return Err(AppError::Unavailable(
                    "tracker_project_context_too_large".into(),
                ));
            }
            bytes.extend_from_slice(&chunk);
        }
        let value: pdlc_shared::project_spaces::ProjectSpaceContextBatch =
            serde_json::from_slice(&bytes)
                .map_err(|_| AppError::Unavailable("invalid_tracker_project_context".into()))?;
        if !value.valid_for(request, chrono::Utc::now().timestamp()) {
            return Err(AppError::Unavailable(
                "tracker_project_context_expired_or_invalid".into(),
            ));
        }
        Ok(value)
    }
    pub(super) async fn projects(
        &self,
    ) -> Result<Vec<shared::resource_context::ResourceContextSummary>, AppError> {
        let registry: Uuid = std::env::var("WIKI_NAMESPACE__REGISTRY_INSTANCE_ID")
            .ok()
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| AppError::Unavailable("namespace_owner_not_configured".into()))?;
        let mut projects = Vec::new();
        let mut namespaces = std::collections::HashSet::new();
        let mut resources = std::collections::HashSet::new();
        for offset in (0..10000).step_by(100) {
            let mut response = self
                .client
                .get(
                    self.endpoint
                        .join("api/v1/namespace-projects")
                        .map_err(AppError::internal)?,
                )
                .query(&[("limit", 100), ("offset", offset)])
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
                .map_err(|_| AppError::Unavailable("invalid_tracker_readback".into()))?
            {
                if bytes.len() + chunk.len() > 262144 {
                    return Err(AppError::Unavailable(
                        "tracker_reader_response_too_large".into(),
                    ));
                }
                bytes.extend_from_slice(&chunk);
            }
            let page: Vec<shared::resource_context::ResourceContextSummary> =
                serde_json::from_slice(&bytes)
                    .map_err(|_| AppError::Unavailable("invalid_tracker_readback".into()))?;
            if page.len() > 100 {
                return Err(AppError::Unavailable("invalid_tracker_project_page".into()));
            }
            let last = page.len() < 100;
            for item in page {
                super::workspace_catalog::validate_project(&item, registry, self.instance)?;
                if !namespaces.insert((
                    item.binding.namespace.registry_instance_id,
                    item.binding.namespace.namespace_id,
                )) || !resources.insert((
                    item.binding.resource.instance_id,
                    item.binding.resource.resource_id,
                )) {
                    return Err(AppError::Unavailable(
                        "duplicate_tracker_project_identity".into(),
                    ));
                }
                projects.push(item);
            }
            if last {
                return Ok(projects);
            }
        }
        Err(AppError::Unavailable(
            "tracker_catalog_limit_exceeded".into(),
        ))
    }
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
        let client = tracker_reader_client(
            reqwest::Client::builder(),
            std::time::Duration::from_secs(10),
        )
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
    async fn task_project(
        &self,
        space: Uuid,
        selected: Option<&NamespaceRef>,
    ) -> Result<NamespaceRef, AppError> {
        let rows = sqlx::query("SELECT registry_instance_id,namespace_id FROM wiki_space_projects WHERE space_id=$1 ORDER BY registry_instance_id,namespace_id")
            .bind(space).fetch_all(&self.pool).await.map_err(AppError::database)?;
        let projects: Vec<NamespaceRef> = rows
            .into_iter()
            .map(|row| NamespaceRef {
                registry_instance_id: row.get("registry_instance_id"),
                namespace_id: row.get("namespace_id"),
            })
            .collect();
        if let Some(project) = selected {
            return projects
                .into_iter()
                .find(|item| item == project)
                .ok_or_else(|| AppError::invalid_input("project_not_linked_to_space"));
        }
        if projects.len() != 1 {
            return Err(AppError::conflict("select_space_project"));
        }
        Ok(projects.into_iter().next().expect("one project"))
    }
    pub(super) async fn namespace_available_tasks(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        selected: Option<&NamespaceRef>,
        offset: u32,
    ) -> Result<Vec<shared::resource_context::TaskCatalogItem>, AppError> {
        let space = self
            .ensure_space_access(claims, space_key, app::wiki::WikiSpaceAccess::Edit)
            .await?;
        self.namespace_binding(space).await?;
        self.project_catalog_actor(claims).await?;
        let namespace = self.task_project(space, selected).await?;
        self.task_reader
            .as_ref()
            .ok_or_else(|| AppError::Unavailable("tracker_reader_not_configured".into()))?
            .list(&namespace, offset)
            .await
    }
    pub(super) async fn namespace_task_revision_links(
        &self,
        namespace: &NamespaceRef,
        task: &TaskRef,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        if self.admission_active().await? {
            return Err(AppError::invalid_input("execution_context_v3_required"));
        }
        let mismatched: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM managed_task_document_links l JOIN task_dossiers t ON t.id=l.dossier_id WHERE t.tracker_instance_id=$1 AND t.task_id=$2 AND (l.registry_instance_id<>$3 OR l.namespace_id<>$4))")
            .bind(task.tracker_instance_id).bind(task.task_id).bind(namespace.registry_instance_id).bind(namespace.namespace_id)
            .fetch_one(&self.pool).await.map_err(AppError::database)?;
        if mismatched {
            return Err(AppError::Forbidden);
        }
        sqlx::query_scalar("SELECT jsonb_build_object('document_id',d.id,'revision_id',r.id,'title',r.title,'version',r.version,'space_key',s.key,'task_key',t.task_key,'namespace',jsonb_build_object('registry_instance_id',l.registry_instance_id,'namespace_id',l.namespace_id)) FROM managed_task_document_links l JOIN task_dossiers t ON t.id=l.dossier_id JOIN documents d ON d.id=l.document_id JOIN document_revisions r ON r.id=l.revision_id JOIN spaces s ON s.id=l.space_id WHERE t.tracker_instance_id=$1 AND t.task_id=$2 AND l.registry_instance_id=$3 AND l.namespace_id=$4 ORDER BY l.created_at DESC,l.id LIMIT 100")
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
        self.namespace_binding(space).await?;
        if let Some(old)=sqlx::query("SELECT l.id,l.dossier_id,l.registry_instance_id,l.namespace_id FROM managed_task_document_links l JOIN task_dossiers t ON t.id=l.dossier_id WHERE l.space_id=$1 AND t.tracker_instance_id=$2 AND t.task_id=$3 AND l.document_id=$4 AND l.revision_id=$5").bind(space).bind(input.task.tracker_instance_id).bind(input.task.task_id).bind(input.document_id).bind(input.revision_id).fetch_optional(&self.pool).await.map_err(AppError::database)? {
            let namespace = NamespaceRef { registry_instance_id: old.get("registry_instance_id"), namespace_id: old.get("namespace_id") };
            if input.namespace.as_ref().is_some_and(|selected| selected != &namespace) { return Err(AppError::conflict("task_ref_namespace_mismatch")); }
            return Ok(serde_json::json!({"id":old.get::<Uuid,_>("id"),"dossier_id":old.get::<Uuid,_>("dossier_id"),"task":input.task,"document_id":input.document_id,"revision_id":input.revision_id}));
        }
        self.project_catalog_actor(claims).await?;
        let namespace = self.task_project(space, input.namespace.as_ref()).await?;
        let reader = self
            .task_reader
            .as_ref()
            .ok_or_else(|| AppError::Unavailable("tracker_reader_not_configured".into()))?;
        let task_key = reader.verify(&namespace, &input.task).await?;
        let actor: Uuid = claims.user_id.parse().map_err(|_| AppError::Unauthorized)?;
        let mut tx = self
            .begin_content_write(
                actor,
                claims.admission.as_ref(),
                &[space],
                app::wiki::WikiSpaceAccess::Edit,
            )
            .await?;
        ensure_document_accepts_writes_tx(&mut tx, input.document_id).await?;
        let linked: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM wiki_space_projects WHERE space_id=$1 AND registry_instance_id=$2 AND namespace_id=$3)")
            .bind(space).bind(namespace.registry_instance_id).bind(namespace.namespace_id)
            .fetch_one(&mut *tx).await.map_err(AppError::database)?;
        if !linked {
            return Err(AppError::conflict("project_not_linked_to_space"));
        }
        let valid: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM document_revisions r JOIN documents d ON d.id=r.document_id WHERE r.id=$1 AND d.id=$2 AND d.space_id=$3)").bind(input.revision_id).bind(input.document_id).bind(space).fetch_one(&mut *tx).await.map_err(AppError::database)?;
        if !valid {
            return Err(AppError::invalid_input("revision_not_in_namespace_space"));
        }
        let dossier: Uuid = sqlx::query_scalar("INSERT INTO task_dossiers(id,space_id,task_key,tracker_instance_id,task_id) VALUES($1,$2,$3,$4,$5) ON CONFLICT(space_id,tracker_instance_id,task_id) WHERE task_id IS NOT NULL DO UPDATE SET task_key=EXCLUDED.task_key,updated_at=now() RETURNING id")
            .bind(Uuid::now_v7()).bind(space).bind(task_key).bind(input.task.tracker_instance_id).bind(input.task.task_id).fetch_one(&mut *tx).await.map_err(AppError::database)?;
        let link: Uuid = sqlx::query_scalar("INSERT INTO managed_task_document_links(id,space_id,dossier_id,document_id,revision_id,created_by,registry_instance_id,namespace_id) VALUES($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT(dossier_id,document_id,revision_id) DO UPDATE SET dossier_id=EXCLUDED.dossier_id RETURNING id")
            .bind(Uuid::now_v7()).bind(space).bind(dossier).bind(input.document_id).bind(input.revision_id).bind(actor).bind(namespace.registry_instance_id).bind(namespace.namespace_id).fetch_one(&mut *tx).await.map_err(AppError::database)?;
        super::write_authorization::commit(tx, claims.admission.as_ref()).await?;
        Ok(
            serde_json::json!({"id":link,"dossier_id":dossier,"task":input.task,"document_id":input.document_id,"revision_id":input.revision_id}),
        )
    }
    pub(super) async fn task_revision_links(
        &self,
        claims: &WikiClaims,
        task: &TaskRef,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let scope = self.read_admission(Some(claims)).await?;
        let visible = scope
            .as_ref()
            .map(|scope| scope.readable_ids())
            .transpose()?;
        let user = self.restricted_user_id(claims).await?;
        // Filter before LIMIT: a private neighboring space must neither leak nor
        // hide documents the caller can read in this shared project.
        let rows = sqlx::query("SELECT l.space_id,jsonb_build_object('id',l.id,'document_id',d.id,'revision_id',r.id,'title',r.title,'version',r.version,'space_key',s.key,'task_key',t.task_key,'namespace',jsonb_build_object('registry_instance_id',l.registry_instance_id,'namespace_id',l.namespace_id)) AS document FROM managed_task_document_links l JOIN task_dossiers t ON t.id=l.dossier_id JOIN documents d ON d.id=l.document_id JOIN document_revisions r ON r.id=l.revision_id JOIN spaces s ON s.id=l.space_id WHERE t.tracker_instance_id=$1 AND t.task_id=$2 AND (($4::uuid[] IS NOT NULL AND s.id=ANY($4)) OR ($4::uuid[] IS NULL AND ($3::uuid IS NULL OR EXISTS(SELECT 1 FROM space_members sm WHERE sm.space_id=s.id AND sm.user_id=$3 AND sm.role IN ('admin','editor','viewer'))))) ORDER BY l.created_at DESC,l.id LIMIT 100")
            .bind(task.tracker_instance_id).bind(task.task_id).bind(user).bind(visible).fetch_all(&self.pool).await.map_err(AppError::database)?;
        let mut result = Vec::new();
        let mut checked = std::collections::BTreeSet::new();
        for row in rows {
            if checked.insert(row.get::<Uuid, _>("space_id")) {
                self.ensure_space_id_access(
                    claims,
                    row.get("space_id"),
                    app::wiki::WikiSpaceAccess::View,
                )
                .await?;
                self.namespace_binding(row.get("space_id")).await?;
            }
            result.push(row.get("document"));
        }
        if let Some(scope) = &scope {
            scope.require_current()?;
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
            client: tracker_reader_client(
                reqwest::Client::builder(),
                std::time::Duration::from_secs(2),
            )
            .unwrap(),
            endpoint: reqwest::Url::parse(&format!("http://{address}/")).unwrap(),
            token: "qa-reader-only".into(),
            instance,
        }
    }

    #[tokio::test]
    async fn tracker_reader_does_not_send_service_token_through_proxy() {
        let proxy = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let proxy_address = proxy.local_addr().unwrap();
        let origin = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin_address = origin.local_addr().unwrap();
        let origin_request = tokio::spawn(async move {
            let (mut stream, _) = origin.accept().await.unwrap();
            let mut request = Vec::new();
            loop {
                let mut chunk = [0_u8; 1024];
                let count = stream.read(&mut chunk).await.unwrap();
                assert_ne!(count, 0);
                request.extend_from_slice(&chunk[..count]);
                if request.windows(4).any(|part| part == b"\r\n\r\n") {
                    break;
                }
            }
            let request = String::from_utf8(request).unwrap();
            assert!(
                request
                    .to_lowercase()
                    .contains("authorization: bearer qa-reader-only")
            );
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .await
                .unwrap();
        });
        let proxy_accept =
            tokio::time::timeout(std::time::Duration::from_millis(250), proxy.accept());
        let client = tracker_reader_client(
            reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(format!("http://{proxy_address}")).unwrap()),
            std::time::Duration::from_secs(2),
        )
        .unwrap();
        client
            .get(format!("http://{origin_address}/tracker"))
            .bearer_auth("qa-reader-only")
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
        origin_request.await.unwrap();
        assert!(
            proxy_accept.await.is_err(),
            "configured proxy must be bypassed"
        );
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
