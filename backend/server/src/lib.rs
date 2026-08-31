use std::{future::IntoFuture, sync::Arc};

use app::WikiAppContext;
use shared::{AppConfig, AppError};
use tokio::sync::oneshot;
use tracing::{error, warn};

/// Maximum time to drain in-flight requests after a shutdown signal.
const SHUTDOWN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

pub async fn run(
    config: Arc<AppConfig>,
    ready: oneshot::Sender<std::net::SocketAddr>,
    shutdown: oneshot::Receiver<()>,
) -> Result<(), AppError> {
    let wiki_backend = persistent_wiki_backend(&config).await?;
    run_with_wiki_backend(config, wiki_backend, ready, shutdown).await
}

pub async fn persistent_wiki_backend(
    config: &AppConfig,
) -> Result<api::routes::wiki::WikiBackend, AppError> {
    let wiki_storage = Arc::new(infra::LocalWikiAttachmentStorage::new(&config.storage.dir));
    let (backend, settings) = infra::connect_postgres_wiki_backend(config, wiki_storage).await?;
    Ok(api::routes::wiki::WikiBackend::persistent(
        backend, settings,
    ))
}

pub async fn run_with_wiki_backend(
    config: Arc<AppConfig>,
    wiki_backend: api::routes::wiki::WikiBackend,
    ready: oneshot::Sender<std::net::SocketAddr>,
    shutdown: oneshot::Receiver<()>,
) -> Result<(), AppError> {
    let ctx = Arc::new(WikiAppContext::new(config.clone()));

    let address = format!("{}:{}", config.server.address, config.server.port);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(AppError::internal)?;
    let bound_addr = listener.local_addr().map_err(AppError::internal)?;
    let _ = ready.send(bound_addr);

    let (shutdown_started_tx, shutdown_started_rx) = tokio::sync::oneshot::channel();
    let server = axum::serve(
        listener,
        api::router_with_wiki(ctx.clone(), wiki_backend).with_state(ctx),
    )
    .with_graceful_shutdown(async move {
        let _ = shutdown.await;
        let _ = shutdown_started_tx.send(());
    })
    .into_future();
    tokio::pin!(server);

    tokio::select! {
        result = &mut server => {
            if let Err(e) = result {
                error!("server error: {e}");
                return Err(AppError::internal(e));
            }
        }
        _ = shutdown_started_rx => {
            if tokio::time::timeout(SHUTDOWN_TIMEOUT, &mut server).await.is_err() {
                warn!("graceful shutdown exceeded {SHUTDOWN_TIMEOUT:?}; dropping active connections");
            }
        }
    }

    Ok(())
}
