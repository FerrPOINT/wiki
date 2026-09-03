use std::{future::IntoFuture, sync::Arc, time::Duration};

use app::WikiAppContext;
use shared::{AppConfig, AppError};
use tokio::{
    sync::oneshot,
    task::JoinHandle,
    time::{MissedTickBehavior, interval},
};
use tracing::{error, info, warn};

/// Maximum time to drain in-flight requests after a shutdown signal.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

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
    let maintenance_task = spawn_maintenance_task(&config, wiki_backend.clone());

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

    let server_error = tokio::select! {
        result = &mut server => {
            if let Err(e) = result {
                error!("server error: {e}");
                Some(AppError::internal(e))
            } else {
                None
            }
        }
        _ = shutdown_started_rx => {
            match tokio::time::timeout(SHUTDOWN_TIMEOUT, &mut server).await {
                Ok(Ok(())) => None,
                Ok(Err(e)) => {
                    error!("server error during shutdown: {e}");
                    Some(AppError::internal(e))
                }
                Err(_) => {
                    warn!("graceful shutdown exceeded {SHUTDOWN_TIMEOUT:?}; dropping active connections");
                    None
                }
            }
        }
    };

    if let Some(task) = maintenance_task {
        task.abort();
        let _ = task.await;
    }

    if let Some(err) = server_error {
        return Err(err);
    }

    Ok(())
}

fn spawn_maintenance_task(
    config: &AppConfig,
    wiki_backend: api::routes::wiki::WikiBackend,
) -> Option<JoinHandle<()>> {
    if !config.maintenance.enabled {
        return None;
    }

    let interval_duration = Duration::from_secs(config.maintenance.interval_seconds);
    Some(tokio::spawn(async move {
        let mut ticks = interval(interval_duration);
        ticks.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            ticks.tick().await;
            match wiki_backend.run_maintenance().await {
                Ok(report) if report.is_empty() => {}
                Ok(report) => {
                    info!(
                        expired_staged_attachments_deleted =
                            report.expired_staged_attachments_deleted,
                        expired_staged_attachment_file_delete_failures =
                            report.expired_staged_attachment_file_delete_failures,
                        expired_idempotency_records_deleted =
                            report.expired_idempotency_records_deleted,
                        "wiki maintenance completed"
                    );
                }
                Err(err) => {
                    warn!(error = %err, "wiki maintenance failed");
                }
            }
        }
    }))
}
