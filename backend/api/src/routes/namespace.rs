use super::wiki::WikiBackend;
use axum::{
    Extension, Json,
    extract::{Path, Query, Request},
    middleware::Next,
    response::Response,
};
use shared::{AppError, resource_context::*};
use uuid::Uuid;

#[utoipa::path(get,operation_id="wiki_space_namespace_context",path="/api/v1/spaces/{space_key}/namespace-context",tag="namespaces",params(("space_key"=String,Path)),responses((status=200,body=Option<ResourceContextSummary>),(status=403,description="Space access denied")))]
pub async fn space_context(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<shared::WikiClaims>,
    Path(key): Path<String>,
) -> Result<Json<Option<ResourceContextSummary>>, AppError> {
    Ok(Json(
        backend
            .namespace_port()?
            .space_namespace_context(&claims, &key)
            .await?,
    ))
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct WorkspaceQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}
#[utoipa::path(get,operation_id="wiki_workspace_projects",path="/api/v1/workspace-projects",tag="namespaces",params(WorkspaceQuery),responses((status=200,body=shared::wiki_contract::WorkspaceProjectPage),(status=503,description="Catalog unavailable")))]
pub async fn workspace_projects(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<shared::WikiClaims>,
    Query(page): Query<WorkspaceQuery>,
) -> Result<Json<shared::wiki_contract::WorkspaceProjectPage>, AppError> {
    Ok(Json(
        backend
            .namespace_port()?
            .workspace_projects(
                &claims,
                None,
                page.limit.unwrap_or(50),
                page.offset.unwrap_or(0),
                page.search.as_deref().unwrap_or(""),
            )
            .await?,
    ))
}
#[utoipa::path(get,operation_id="wiki_workspace_project",path="/api/v1/workspace-projects/{registry}/{namespace}",tag="namespaces",params(("registry"=Uuid,Path),("namespace"=Uuid,Path)),responses((status=200,body=shared::wiki_contract::WorkspaceProject),(status=404,description="Project not found")))]
pub async fn workspace_project(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<shared::WikiClaims>,
    Path((registry, namespace)): Path<(Uuid, Uuid)>,
) -> Result<Json<shared::wiki_contract::WorkspaceProject>, AppError> {
    if registry.is_nil() || namespace.is_nil() {
        return Err(AppError::invalid_input("invalid_project_ref"));
    }
    let page = backend
        .namespace_port()?
        .workspace_projects(
            &claims,
            Some(&NamespaceRef {
                registry_instance_id: registry,
                namespace_id: namespace,
            }),
            1,
            0,
            "",
        )
        .await?;
    Ok(Json(page.items.into_iter().next().ok_or_else(|| {
        AppError::not_found("workspace_project", namespace)
    })?))
}
#[utoipa::path(get,operation_id="wiki_document_namespace_context",path="/api/v1/documents/{id}/namespace-context",tag="namespaces",params(("id"=Uuid,Path)),responses((status=200,body=Option<ResourceContextSummary>),(status=403,description="Document access denied")))]
pub async fn document_context(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<shared::WikiClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Option<ResourceContextSummary>>, AppError> {
    Ok(Json(
        backend
            .namespace_port()?
            .document_namespace_context(&claims, id)
            .await?,
    ))
}

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
    pub registry_instance_id: Option<Uuid>,
    pub namespace_id: Option<Uuid>,
    pub offset: Option<u32>,
}
#[utoipa::path(get,operation_id="wiki_available_task_refs",path="/api/v1/spaces/{space_key}/available-tasks",tag="namespaces",params(("space_key"=String,Path),TaskPage),responses((status=200,body=Vec<TaskCatalogItem>)))]
pub async fn available_tasks(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<shared::WikiClaims>,
    Path(key): Path<String>,
    Query(page): Query<TaskPage>,
) -> Result<Json<Vec<TaskCatalogItem>>, AppError> {
    let namespace = match (page.registry_instance_id, page.namespace_id) {
        (Some(registry_instance_id), Some(namespace_id)) => Some(NamespaceRef {
            registry_instance_id,
            namespace_id,
        }),
        (None, None) => None,
        _ => return Err(AppError::invalid_input("incomplete_project_ref")),
    };
    Ok(Json(
        backend
            .namespace_port()?
            .namespace_available_tasks(&claims, &key, namespace.as_ref(), page.offset.unwrap_or(0))
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

fn may_apply_owner_v2(ctx: &pdlc_auth_core::AuthContext, registered: bool) -> bool {
    ctx.session_id.is_none() && registered && ctx.has_scope("wiki:write")
}
pub async fn owner_v2_auth(mut req: Request, next: Next) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?
        .to_owned();
    let verified = infra::wiki_postgres::central_auth::BRIDGE
        .try_verified_token(&token)
        .await
        .map_err(|e| match e {
            pdlc_auth_core::service_bridge::VerifiedBridgeError::Unavailable
            | pdlc_auth_core::service_bridge::VerifiedBridgeError::NotConfigured => {
                AppError::Unavailable("central_owner_confirmation_unavailable".into())
            }
            _ => AppError::Unauthorized,
        })?;
    let registered = std::env::var("WIKI_NAMESPACE__OWNER_SUBJECTS")
        .unwrap_or_default()
        .split(',')
        .any(|s| !s.trim().is_empty() && s.trim() == verified.context.user_id);
    if !may_apply_owner_v2(&verified.context, registered) {
        return Err(AppError::Forbidden);
    }
    let validity = pdlc_shared::space_access::AdmissionValidity::confirmed(
        verified.lifetime.source_observed_at,
        verified.lifetime.local_valid_until(),
        i64::MAX,
    )
    .ok_or(AppError::Unauthorized)?;
    let authority = shared::space_owner::WikiSpaceOwnerAuthority::confirmed(
        pdlc_shared::space_access::CentralIdentity {
            issuer: verified.issuer,
            subject: verified.context.user_id,
        },
        token,
        validity,
    )?;
    req.extensions_mut().insert(authority);
    Ok(next.run(req).await)
}
#[utoipa::path(put,operation_id="wiki_space_owner_v2",path="/api/v2/spaces/{registry}/{space}/owner-state",tag="spaces-v2",params(("registry"=Uuid,Path),("space"=Uuid,Path)),request_body=pdlc_shared::space_lifecycle::SpaceOwnerCommandV2,responses((status=200,body=pdlc_shared::space_lifecycle::SpaceOwnerReadbackV2),(status=403,description="Registered owner PAT with exact wiki write scope required"),(status=409,description="Original mapping/revision/operation conflict")))]
pub async fn apply_space_v2(
    Extension(backend): Extension<WikiBackend>,
    Extension(authority): Extension<shared::space_owner::WikiSpaceOwnerAuthority>,
    Path((registry, space)): Path<(Uuid, Uuid)>,
    Json(command): Json<pdlc_shared::space_lifecycle::SpaceOwnerCommandV2>,
) -> Result<Json<pdlc_shared::space_lifecycle::SpaceOwnerReadbackV2>, AppError> {
    if registry != command.space.registry_instance_id || space != command.space.space_id {
        return Err(AppError::invalid_input("space_ref_mismatch"));
    }
    let result = backend
        .namespace_port()?
        .apply_space_owner_v2(&command, &authority)
        .await?;
    authority.require_current()?;
    Ok(Json(result))
}
#[cfg(test)]
mod owner_v2_tests {
    use super::*;
    fn context(session: bool, scope: &str) -> pdlc_auth_core::AuthContext {
        pdlc_auth_core::AuthContext {
            user_id: "owner-subject".into(),
            email: None,
            role: Some("admin".into()),
            scopes: std::collections::HashSet::from([scope.to_owned()]),
            session_id: session.then(|| "browser".into()),
            token: "synthetic".into(),
        }
    }
    #[test]
    fn owner_v2_needs_registered_machine_and_exact_scope_not_browser_admin() {
        assert!(may_apply_owner_v2(&context(false, "wiki:write"), true));
        assert!(!may_apply_owner_v2(&context(true, "wiki:write"), true));
        assert!(!may_apply_owner_v2(&context(false, "wiki:write"), false));
        assert!(!may_apply_owner_v2(&context(false, "wiki:read"), true));
        assert!(!may_apply_owner_v2(
            &context(false, "task-tracker:write"),
            true
        ));
    }
}

fn may_apply_project_projection(ctx: &pdlc_auth_core::AuthContext, registered: bool) -> bool {
    ctx.session_id.is_none() && registered && ctx.has_scope("wiki:write")
}
pub async fn project_projection_auth(mut req: Request, next: Next) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;
    let verified = infra::wiki_postgres::central_auth::BRIDGE
        .try_verified_token(token)
        .await
        .map_err(|_| AppError::Unavailable("project_owner_confirmation_unavailable".into()))?;
    let registered = std::env::var("WIKI_PROJECTS__TRACKER_SUBJECTS")
        .unwrap_or_default()
        .split(',')
        .any(|s| !s.trim().is_empty() && s.trim() == verified.context.user_id);
    if !may_apply_project_projection(&verified.context, registered) {
        return Err(AppError::Forbidden);
    }
    let validity = pdlc_shared::space_access::AdmissionValidity::confirmed(
        verified.lifetime.source_observed_at,
        verified.lifetime.local_valid_until(),
        i64::MAX,
    )
    .ok_or(AppError::Unauthorized)?;
    let proof = pdlc_shared::space_lifecycle::SpaceOwnerAuthority::confirmed(
        pdlc_shared::space_access::CentralIdentity {
            issuer: verified.issuer,
            subject: verified.context.user_id,
        },
        validity,
    )
    .ok_or(AppError::Unauthorized)?;
    req.extensions_mut().insert(proof);
    Ok(next.run(req).await)
}
#[utoipa::path(put,operation_id="wiki_project_spaces_projection",path="/api/v2/projects/{tracker}/{project}/spaces/projection",tag="spaces-v2",params(("tracker"=Uuid,Path),("project"=Uuid,Path)),request_body=pdlc_shared::project_spaces::ProjectSpacesProjection,responses((status=200,body=pdlc_shared::project_spaces::ProjectSpacesProjectionReadback),(status=409,description="Original operation, mapping or revision conflict")))]
pub async fn apply_project_projection(
    Extension(backend): Extension<WikiBackend>,
    Extension(proof): Extension<pdlc_shared::space_lifecycle::SpaceOwnerAuthority>,
    Path((tracker, project)): Path<(Uuid, Uuid)>,
    Json(command): Json<pdlc_shared::project_spaces::ProjectSpacesProjection>,
) -> Result<Json<pdlc_shared::project_spaces::ProjectSpacesProjectionReadback>, AppError> {
    if command.project.tracker_instance_id != tracker || command.project.project_id != project {
        return Err(AppError::invalid_input("project_ref_mismatch"));
    }
    let ack = backend
        .namespace_port()?
        .apply_project_spaces_projection(&command, &proof)
        .await?;
    if !proof.is_current() {
        return Err(AppError::Forbidden);
    }
    Ok(Json(ack))
}
#[cfg(test)]
mod projection_auth_tests {
    use super::*;
    #[test]
    fn projection_needs_registered_tracker_pat_not_an_admin_session() {
        let mut ctx = pdlc_auth_core::AuthContext {
            user_id: "tracker-machine".into(),
            role: Some("admin".into()),
            email: None,
            scopes: std::collections::HashSet::from(["wiki:write".into()]),
            session_id: None,
            token: String::new(),
        };
        assert!(may_apply_project_projection(&ctx, true));
        assert!(!may_apply_project_projection(&ctx, false));
        ctx.session_id = Some("admin-browser".into());
        assert!(!may_apply_project_projection(&ctx, true));
        ctx.session_id = None;
        ctx.scopes = std::collections::HashSet::from(["wiki:read".into()]);
        assert!(!may_apply_project_projection(&ctx, true));
    }
}
