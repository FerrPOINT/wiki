use super::wiki::WikiBackend;
use axum::{
    Extension, Json,
    extract::{Path, Query, Request},
    middleware::Next,
    response::Response,
};
use shared::{AppError, resource_context::*};
use uuid::Uuid;

pub async fn owner_auth(req: Request, next: Next) -> Result<Response, AppError> {
    machine_auth(req, next, "WIKI_NAMESPACE__OWNER_SUBJECTS").await
}
pub async fn reader_auth(req: Request, next: Next) -> Result<Response, AppError> {
    machine_auth(req, next, "WIKI_NAMESPACE__READER_SUBJECTS").await
}
async fn machine_auth(req: Request, next: Next, key: &str) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;
    let ctx = infra::wiki_postgres::central_auth::try_central(token)
        .await?
        .ok_or(AppError::Unauthorized)?;
    let registered = std::env::var(key)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .any(|s| !s.is_empty() && s == ctx.user_id);
    if ctx.session_id.is_some() || !registered || !ctx.allows_service("wiki", req.method().as_str())
    {
        return Err(AppError::Forbidden);
    }
    Ok(next.run(req).await)
}

#[utoipa::path(put,operation_id="wiki_namespace_apply",path="/api/v1/namespace-resources/wiki_space/{id}",tag="namespaces",params(("id"=Uuid,Path)),request_body=OwnerCommand,responses((status=200,body=OwnerReadback),(status=409,description="Binding/fence conflict")))]
pub async fn apply(
    Extension(backend): Extension<WikiBackend>,
    Path(id): Path<Uuid>,
    Json(command): Json<OwnerCommand>,
) -> Result<Json<OwnerReadback>, AppError> {
    if id != command.resource.resource_id {
        return Err(AppError::invalid_input("resource_id_mismatch"));
    }
    Ok(Json(
        backend.namespace_port()?.apply_namespace(&command).await?,
    ))
}
#[utoipa::path(get,operation_id="wiki_namespace_readback",path="/api/v1/namespace-resources/wiki_space/{id}",tag="namespaces",params(("id"=Uuid,Path)),responses((status=200,body=OwnerReadback),(status=404,description="Binding not found")))]
pub async fn readback(
    Extension(backend): Extension<WikiBackend>,
    Path(id): Path<Uuid>,
) -> Result<Json<OwnerReadback>, AppError> {
    Ok(Json(
        backend
            .namespace_port()?
            .namespace_binding(id)
            .await?
            .ok_or_else(|| AppError::not_found("namespace_binding", id))?,
    ))
}

#[utoipa::path(post,operation_id="wiki_namespace_link_revision",path="/api/v1/spaces/{space_key}/managed-task-links",tag="namespaces",params(("space_key"=String,Path)),request_body=shared::managed_links::LinkTaskRevision,responses((status=200,description="Verified TaskRef and immutable revision link"),(status=503,description="Tracker reader unavailable")))]
pub async fn link_revision(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<shared::WikiClaims>,
    Path(key): Path<String>,
    Json(input): Json<shared::managed_links::LinkTaskRevision>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(
        backend
            .namespace_port()?
            .link_task_revision(&claims, &key, &input)
            .await?,
    ))
}
#[utoipa::path(get,operation_id="wiki_namespace_revision_links",path="/api/v1/managed-task-links/{tracker_instance}/{task}",tag="namespaces",params(("tracker_instance"=Uuid,Path),("task"=Uuid,Path)),responses((status=200,body=Vec<shared::managed_links::TaskRevisionLinkResponse>)))]
pub async fn revision_links(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<shared::WikiClaims>,
    Path((instance, task)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<shared::managed_links::TaskRevisionLinkResponse>>, AppError> {
    Ok(Json(typed_links(
        backend
            .namespace_port()?
            .task_revision_links(
                &claims,
                &shared::managed_links::TaskRef {
                    tracker_instance_id: instance,
                    task_id: task,
                },
            )
            .await?,
    )?))
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct Page {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct TaskPage {
    pub offset: Option<u32>,
}
#[utoipa::path(get,operation_id="wiki_available_task_refs",path="/api/v1/spaces/{space_key}/available-tasks",tag="namespaces",params(("space_key"=String,Path),TaskPage),responses((status=200,body=Vec<TaskCatalogItem>)))]
pub async fn available_tasks(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<shared::WikiClaims>,
    Path(key): Path<String>,
    Query(page): Query<TaskPage>,
) -> Result<Json<Vec<TaskCatalogItem>>, AppError> {
    Ok(Json(
        backend
            .namespace_port()?
            .namespace_available_tasks(&claims, &key, page.offset.unwrap_or(0))
            .await?,
    ))
}
#[utoipa::path(get,operation_id="wiki_namespace_available_resources",path="/api/v1/namespace-available-resources",tag="namespaces",params(Page),responses((status=200,body=Vec<ResourceCatalogItem>)))]
pub async fn available_resources(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<shared::WikiClaims>,
    Query(page): Query<Page>,
) -> Result<Json<Vec<ResourceCatalogItem>>, AppError> {
    Ok(Json(
        backend
            .namespace_port()?
            .namespace_available_resources(
                &claims,
                page.limit.unwrap_or(50),
                page.offset.unwrap_or(0),
            )
            .await?,
    ))
}
#[utoipa::path(get,operation_id="wiki_namespace_stats",path="/api/v1/namespace-stats/{registry}/{namespace}",tag="namespaces",params(("registry"=Uuid,Path),("namespace"=Uuid,Path)),responses((status=200,body=ResourceStats)))]
pub async fn stats(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<shared::WikiClaims>,
    Path((registry, namespace)): Path<(Uuid, Uuid)>,
) -> Result<Json<ResourceStats>, AppError> {
    Ok(Json(
        backend
            .namespace_port()?
            .namespace_stats(
                &claims,
                &NamespaceRef {
                    registry_instance_id: registry,
                    namespace_id: namespace,
                },
            )
            .await?,
    ))
}
#[utoipa::path(get,operation_id="wiki_namespace_contexts",path="/api/v1/namespace-contexts",tag="namespaces",params(Page),responses((status=200,body=Vec<ResourceContextSummary>)))]
pub async fn contexts(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<shared::WikiClaims>,
    Query(page): Query<Page>,
) -> Result<Json<Vec<ResourceContextSummary>>, AppError> {
    Ok(Json(
        backend
            .namespace_port()?
            .namespace_contexts(
                &claims,
                None,
                page.limit.unwrap_or(50),
                page.offset.unwrap_or(0),
            )
            .await?,
    ))
}
#[utoipa::path(get,operation_id="wiki_namespace_context",path="/api/v1/namespace-contexts/{registry}/{namespace}",tag="namespaces",params(("registry"=Uuid,Path),("namespace"=Uuid,Path)),responses((status=200,body=ResourceContextSummary),(status=404,description="No confirmed local binding")))]
pub async fn context(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<shared::WikiClaims>,
    Path((registry, namespace)): Path<(Uuid, Uuid)>,
) -> Result<Json<ResourceContextSummary>, AppError> {
    let mut resources = backend
        .namespace_port()?
        .namespace_contexts(
            &claims,
            Some(&NamespaceRef {
                registry_instance_id: registry,
                namespace_id: namespace,
            }),
            1,
            0,
        )
        .await?;
    Ok(Json(resources.pop().ok_or_else(|| {
        AppError::not_found("namespace_binding", namespace)
    })?))
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct NamespaceQuery {
    pub registry_instance_id: Uuid,
    pub namespace_id: Uuid,
}
#[utoipa::path(get,operation_id="wiki_namespace_stored_links",path="/api/v1/namespace-task-links/{tracker_instance}/{task}",tag="namespaces",params(("tracker_instance"=Uuid,Path),("task"=Uuid,Path),NamespaceQuery),responses((status=200,body=Vec<shared::managed_links::TaskRevisionLinkResponse>)))]
pub async fn stored_links(
    Extension(backend): Extension<WikiBackend>,
    Path((instance, task)): Path<(Uuid, Uuid)>,
    Query(namespace): Query<NamespaceQuery>,
) -> Result<Json<Vec<shared::managed_links::TaskRevisionLinkResponse>>, AppError> {
    Ok(Json(typed_links(
        backend
            .namespace_port()?
            .namespace_task_revision_links(
                &NamespaceRef {
                    registry_instance_id: namespace.registry_instance_id,
                    namespace_id: namespace.namespace_id,
                },
                &shared::managed_links::TaskRef {
                    tracker_instance_id: instance,
                    task_id: task,
                },
            )
            .await?,
    )?))
}

fn typed_links(
    values: Vec<serde_json::Value>,
) -> Result<Vec<shared::managed_links::TaskRevisionLinkResponse>, AppError> {
    values
        .into_iter()
        .map(|value| {
            serde_json::from_value(value)
                .map_err(|_| AppError::Unavailable("invalid_stored_revision_link".into()))
        })
        .collect()
}
