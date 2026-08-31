use axum::{
    Extension, Router,
    http::{HeaderName, HeaderValue, Method},
    middleware::from_fn_with_state,
    routing::{get, post, put},
};
use axum_prometheus::{GenericMetricLayer, PrometheusMetricLayer};
use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::{Arc, OnceLock};
use tower::ServiceBuilder;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    cors::{Any, CorsLayer},
    set_header::SetResponseHeaderLayer,
};
use utoipa::{
    Modify, OpenApi,
    openapi::{
        License,
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    },
};
use utoipa_swagger_ui::SwaggerUi;

/// Global Prometheus metrics handle, initialized once and reused across router builds.
static METRIC_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();
const OPENAPI_LICENSE_NAME: &str = "FerrPOINT Proprietary Source-Available Evaluation License v1.0";
const OPENAPI_LICENSE_URL: &str = "./LICENSE";

fn metric_handle() -> PrometheusHandle {
    METRIC_HANDLE
        .get_or_init(|| {
            let recorder = metrics_exporter_prometheus::PrometheusBuilder::new().build_recorder();
            let handle = recorder.handle();
            metrics::set_global_recorder(Box::new(recorder))
                .expect("failed to set global metrics recorder");
            handle
        })
        .clone()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FallbackIpKeyExtractor;

impl tower_governor::key_extractor::KeyExtractor for FallbackIpKeyExtractor {
    type Key = std::net::IpAddr;

    fn extract<T>(
        &self,
        req: &axum::http::Request<T>,
    ) -> Result<Self::Key, tower_governor::GovernorError> {
        Ok(tower_governor::key_extractor::SmartIpKeyExtractor
            .extract(req)
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)))
    }
}

pub mod routes;

pub use routes::*;

mod wiki_postgres;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::health,
        routes::wiki::register,
        routes::wiki::login,
        routes::wiki::refresh,
        routes::wiki::logout,
        routes::wiki::get_current_user,
        routes::wiki::get_settings,
        routes::wiki::list_users,
        routes::wiki::create_user,
        routes::wiki::update_user,
        routes::wiki::list_spaces,
        routes::wiki::create_space,
        routes::wiki::get_space,
        routes::wiki::update_space,
        routes::wiki::archive_space,
        routes::wiki::list_space_members,
        routes::wiki::upsert_space_member,
        routes::wiki::delete_space_member,
        routes::wiki::get_space_tree,
        routes::wiki::create_document,
        routes::wiki::get_document,
        routes::wiki::update_document_draft,
        routes::wiki::publish_document,
        routes::wiki::archive_document,
        routes::wiki::move_document,
        routes::wiki::list_document_revisions,
        routes::wiki::get_document_revision,
        routes::wiki::list_tasks,
        routes::wiki::get_task,
        routes::wiki::link_task_document,
        routes::wiki::list_task_documents,
        routes::wiki::list_task_evidence,
        routes::wiki::list_phases,
        routes::wiki::get_phase,
        routes::wiki::link_phase_document,
        routes::wiki::list_phase_documents,
        routes::wiki::list_phase_evidence,
        routes::wiki::create_evidence,
        routes::wiki::list_evidence,
        routes::wiki::get_evidence,
        routes::wiki::upload_attachment,
        routes::wiki::get_attachment,
        routes::wiki::download_attachment,
        routes::wiki::list_templates,
        routes::wiki::create_template,
        routes::wiki::list_audit_log,
        routes::wiki::search
    ),
    components(schemas(
        routes::wiki::WikiAuthResponse,
        routes::wiki::WikiRegisterRequest,
        routes::wiki::WikiLoginRequest,
        routes::wiki::WikiRefreshRequest,
        routes::wiki::WikiUserResponse,
        routes::wiki::WikiUserListResponse,
        routes::wiki::WikiCreateUserRequest,
        routes::wiki::WikiUpdateUserRequest,
        routes::wiki::WikiSettingsResponse,
        routes::wiki::SpaceResponse,
        routes::wiki::SpaceListResponse,
        routes::wiki::CreateSpaceRequest,
        routes::wiki::UpdateSpaceRequest,
        routes::wiki::SpaceMemberResponse,
        routes::wiki::SpaceMemberListResponse,
        routes::wiki::UpsertSpaceMemberRequest,
        routes::wiki::SpaceTreeNodeResponse,
        routes::wiki::SpaceTreeResponse,
        routes::wiki::CreateDocumentRequest,
        routes::wiki::UpdateDocumentDraftRequest,
        routes::wiki::PublishDocumentRequest,
        routes::wiki::MoveDocumentRequest,
        routes::wiki::LinkDocumentRequest,
        routes::wiki::DocumentResponse,
        routes::wiki::DocumentSummaryResponse,
        routes::wiki::DocumentListResponse,
        routes::wiki::DocumentRevisionResponse,
        routes::wiki::DocumentRevisionListResponse,
        routes::wiki::TaskPageResponse,
        routes::wiki::TaskPageListResponse,
        routes::wiki::PhasePageResponse,
        routes::wiki::PhasePageListResponse,
        routes::wiki::CreateEvidenceRequest,
        routes::wiki::EvidenceResponse,
        routes::wiki::EvidenceListResponse,
        routes::wiki::AttachmentResponse,
        routes::wiki::TemplateResponse,
        routes::wiki::TemplateListResponse,
        routes::wiki::CreateTemplateRequest,
        routes::wiki::AuditEntryResponse,
        routes::wiki::AuditLogResponse,
        routes::wiki::SearchResultResponse,
        routes::wiki::SearchResponse
    )),
    tags(
        (name = "health", description = "Runtime health"),
        (name = "auth", description = "Authentication"),
        (name = "users", description = "Users and roles"),
        (name = "spaces", description = "Wiki spaces and members"),
        (name = "documents", description = "Documents, tree and revisions"),
        (name = "tasks", description = "Task dossiers by external task key"),
        (name = "phases", description = "Workflow phase dossiers by phase key"),
        (name = "evidence", description = "Links and file evidence"),
        (name = "attachments", description = "Attachment metadata and download"),
        (name = "templates", description = "Document templates"),
        (name = "settings", description = "Instance settings"),
        (name = "audit", description = "Audit log"),
        (name = "search", description = "Search")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let mut license = License::new(OPENAPI_LICENSE_NAME);
        license.url = Some(OPENAPI_LICENSE_URL.to_string());
        openapi.info.license = Some(license);

        let components = openapi
            .components
            .get_or_insert_with(utoipa::openapi::Components::new);
        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

pub fn router_for_memory_tests(ctx: Arc<app::WikiAppContext>) -> Router<Arc<app::WikiAppContext>> {
    let wiki_backend = routes::wiki::WikiBackend::memory_from_config(&ctx.config);
    router_with_wiki(ctx, wiki_backend)
}

pub fn router_with_wiki(
    ctx: Arc<app::WikiAppContext>,
    wiki_backend: routes::wiki::WikiBackend,
) -> Router<Arc<app::WikiAppContext>> {
    let cors = cors_layer(&ctx.config.server.cors_allowed_origins);

    let auth_limiter = GovernorConfigBuilder::default()
        .key_extractor(FallbackIpKeyExtractor)
        .period(std::time::Duration::from_secs(
            ctx.config.server.auth_rate_period_secs,
        ))
        .burst_size(ctx.config.server.auth_rate_burst)
        .finish()
        .expect("valid auth rate limit config");

    let general_limiter = GovernorConfigBuilder::default()
        .key_extractor(FallbackIpKeyExtractor)
        .period(std::time::Duration::from_secs(
            ctx.config.server.general_rate_period_secs,
        ))
        .burst_size(ctx.config.server.general_rate_burst)
        .finish()
        .expect("valid general rate limit config");

    let public =
        Router::<Arc<app::WikiAppContext>>::new().route("/health", get(routes::health::health));

    let auth_routes = Router::<Arc<app::WikiAppContext>>::new()
        .route("/auth/register", post(routes::wiki::register))
        .route("/auth/login", post(routes::wiki::login))
        .route("/auth/refresh", post(routes::wiki::refresh))
        .layer(Extension(wiki_backend.clone()))
        .layer(GovernorLayer::new(auth_limiter));

    let protected = Router::<Arc<app::WikiAppContext>>::new()
        .route("/auth/logout", post(routes::wiki::logout))
        .route("/users/me", get(routes::wiki::get_current_user))
        .route("/settings", get(routes::wiki::get_settings))
        .route(
            "/users",
            get(routes::wiki::list_users).post(routes::wiki::create_user),
        )
        .route("/users/{user_id}", put(routes::wiki::update_user))
        .route(
            "/spaces",
            get(routes::wiki::list_spaces).post(routes::wiki::create_space),
        )
        .route(
            "/spaces/{space_key}",
            get(routes::wiki::get_space).put(routes::wiki::update_space),
        )
        .route(
            "/spaces/{space_key}/archive",
            post(routes::wiki::archive_space),
        )
        .route(
            "/spaces/{space_key}/members",
            get(routes::wiki::list_space_members),
        )
        .route(
            "/spaces/{space_key}/members/{user_id}",
            put(routes::wiki::upsert_space_member).delete(routes::wiki::delete_space_member),
        )
        .route(
            "/spaces/{space_key}/tree",
            get(routes::wiki::get_space_tree),
        )
        .route(
            "/spaces/{space_key}/documents",
            post(routes::wiki::create_document),
        )
        .route("/documents/{document_id}", get(routes::wiki::get_document))
        .route(
            "/documents/{document_id}/draft",
            put(routes::wiki::update_document_draft),
        )
        .route(
            "/documents/{document_id}/publish",
            post(routes::wiki::publish_document),
        )
        .route(
            "/documents/{document_id}/archive",
            post(routes::wiki::archive_document),
        )
        .route(
            "/documents/{document_id}/move",
            post(routes::wiki::move_document),
        )
        .route(
            "/documents/{document_id}/revisions",
            get(routes::wiki::list_document_revisions),
        )
        .route(
            "/documents/{document_id}/revisions/{revision_id}",
            get(routes::wiki::get_document_revision),
        )
        .route("/spaces/{space_key}/tasks", get(routes::wiki::list_tasks))
        .route(
            "/spaces/{space_key}/tasks/{task_key}",
            get(routes::wiki::get_task),
        )
        .route(
            "/spaces/{space_key}/tasks/{task_key}/links/documents",
            post(routes::wiki::link_task_document),
        )
        .route(
            "/spaces/{space_key}/tasks/{task_key}/documents",
            get(routes::wiki::list_task_documents),
        )
        .route(
            "/spaces/{space_key}/tasks/{task_key}/evidence",
            get(routes::wiki::list_task_evidence),
        )
        .route("/spaces/{space_key}/phases", get(routes::wiki::list_phases))
        .route(
            "/spaces/{space_key}/phases/{phase_key}",
            get(routes::wiki::get_phase),
        )
        .route(
            "/spaces/{space_key}/phases/{phase_key}/links/documents",
            post(routes::wiki::link_phase_document),
        )
        .route(
            "/spaces/{space_key}/phases/{phase_key}/documents",
            get(routes::wiki::list_phase_documents),
        )
        .route(
            "/spaces/{space_key}/phases/{phase_key}/evidence",
            get(routes::wiki::list_phase_evidence),
        )
        .route(
            "/evidence",
            get(routes::wiki::list_evidence).post(routes::wiki::create_evidence),
        )
        .route("/evidence/{evidence_id}", get(routes::wiki::get_evidence))
        .route("/attachments", post(routes::wiki::upload_attachment))
        .route(
            "/attachments/{attachment_id}",
            get(routes::wiki::get_attachment),
        )
        .route(
            "/attachments/{attachment_id}/download",
            get(routes::wiki::download_attachment),
        )
        .route(
            "/templates",
            get(routes::wiki::list_templates).post(routes::wiki::create_template),
        )
        .route("/audit-log", get(routes::wiki::list_audit_log))
        .route("/search", get(routes::wiki::search))
        .route_layer(from_fn_with_state(
            wiki_backend.clone(),
            routes::wiki::require_wiki_auth,
        ))
        .layer(Extension(wiki_backend));

    let api = public.merge(auth_routes).merge(protected);
    let handle = metric_handle();
    let prometheus_layer: PrometheusMetricLayer = GenericMetricLayer::new();

    let app = Router::<Arc<app::WikiAppContext>>::new()
        .route("/metrics", get(move || std::future::ready(handle.render())))
        .nest("/api/v1", api.layer(GovernorLayer::new(general_limiter)))
        .merge(SwaggerUi::new("/swagger-ui").url("/api/v1/openapi.json", ApiDoc::openapi()));

    with_security_headers(app)
        .layer(prometheus_layer)
        .layer(cors)
}

fn cors_layer(origins: &[String]) -> CorsLayer {
    if origins.len() == 1 && origins[0] == "*" {
        return CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_origin(Any)
            .allow_headers(Any);
    }

    let origins: Vec<HeaderValue> = origins
        .iter()
        .filter(|origin| !origin.is_empty())
        .map(|origin| {
            origin
                .parse::<HeaderValue>()
                .expect("invalid cors allowed origin")
        })
        .collect();
    CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(tower_http::cors::AllowOrigin::list(origins))
        .allow_headers(Any)
}

fn with_security_headers<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.layer(
        ServiceBuilder::new()
            .layer(SetResponseHeaderLayer::overriding(
                axum::http::header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("x-xss-protection"),
                HeaderValue::from_static("1; mode=block"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("no-referrer"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("content-security-policy"),
                HeaderValue::from_static(
                    "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; font-src 'self'",
                ),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("strict-transport-security"),
                HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            )),
    )
}
