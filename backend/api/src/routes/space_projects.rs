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

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct CanonicalPage {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}
#[utoipa::path(get,operation_id="wiki_canonical_space_projects",path="/api/v2/spaces/{space_key}/projects",tag="spaces-v2",params(("space_key"=String,Path),CanonicalPage),responses((status=200,body=shared::wiki_contract::SpaceProjectsV2),(status=403,description="Original space admission required"),(status=503,description="Fresh canonical owner context unavailable")))]
pub async fn canonical_list(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Path(key): Path<String>,
    axum::extract::Query(page): axum::extract::Query<CanonicalPage>,
) -> Result<axum::response::Response, AppError> {
    let value = backend
        .namespace_port()?
        .canonical_space_projects(
            &claims,
            &key,
            page.limit.unwrap_or(50),
            page.offset.unwrap_or(0),
        )
        .await?;
    if !value
        .validity
        .valid_at(chrono::Utc::now().timestamp(), i64::MAX)
    {
        return Err(AppError::Unavailable(
            "space_project_context_expired".into(),
        ));
    }
    use axum::response::IntoResponse;
    let scope = claims.admission.as_ref().ok_or(AppError::Forbidden)?;
    let until = value.validity.valid_until;
    let mut response = Json(value).into_response();
    super::space_response::restrict(&mut response, scope, until)?;
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("private, no-store"),
    );
    Ok(response)
}
