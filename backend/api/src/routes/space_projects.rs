use super::wiki::WikiBackend;
use axum::{Extension, Json, extract::Path, http::StatusCode};
use shared::{
    AppError, WikiClaims, resource_context::NamespaceRef, wiki_contract::SpaceProjectsResponse,
};
use uuid::Uuid;

#[utoipa::path(get, operation_id="wiki_space_projects", path="/api/v1/spaces/{space_key}/projects", tag="spaces",
    params(("space_key"=String,Path)), responses((status=200,body=SpaceProjectsResponse),(status=403,description="Space or project catalog access denied")))]
pub async fn list(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Path(key): Path<String>,
) -> Result<Json<SpaceProjectsResponse>, AppError> {
    Ok(Json(
        backend
            .namespace_port()?
            .space_projects(&claims, &key)
            .await?,
    ))
}

#[utoipa::path(get, operation_id="wiki_document_space_context", path="/api/v1/documents/{id}/space-context", tag="spaces",
    params(("id"=Uuid,Path)), responses((status=200,body=SpaceProjectsResponse),(status=403,description="Space access denied")))]
pub async fn document_context(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<SpaceProjectsResponse>, AppError> {
    Ok(Json(
        backend
            .namespace_port()?
            .document_space_projects(&claims, id)
            .await?,
    ))
}

#[utoipa::path(put, operation_id="wiki_link_space_project", path="/api/v1/spaces/{space_key}/projects/{registry}/{namespace}", tag="spaces",
    params(("space_key"=String,Path),("registry"=Uuid,Path),("namespace"=Uuid,Path)),
    responses((status=204,description="Project associated with space; repeated link is a no-op"),(status=403,description="Space management denied"),(status=409,description="Archived project")))]
pub async fn link(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Path((key, registry_instance_id, namespace_id)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    backend
        .namespace_port()?
        .link_space_project(
            &claims,
            &key,
            &NamespaceRef {
                registry_instance_id,
                namespace_id,
            },
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(delete, operation_id="wiki_unlink_space_project", path="/api/v1/spaces/{space_key}/projects/{registry}/{namespace}", tag="spaces",
    params(("space_key"=String,Path),("registry"=Uuid,Path),("namespace"=Uuid,Path)),
    responses((status=204,description="Association removed; space and documents preserved"),(status=403,description="Space management denied")))]
pub async fn unlink(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Path((key, registry_instance_id, namespace_id)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    backend
        .namespace_port()?
        .unlink_space_project(
            &claims,
            &key,
            &NamespaceRef {
                registry_instance_id,
                namespace_id,
            },
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
