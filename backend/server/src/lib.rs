use std::{future::IntoFuture, sync::Arc};

use app::AppContext;
use shared::AppConfig;
use tokio::sync::oneshot;
use tracing::{error, warn};

/// Maximum time to drain in-flight requests after a shutdown signal.
const SHUTDOWN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

pub async fn run(
    config: Arc<AppConfig>,
    ready: oneshot::Sender<std::net::SocketAddr>,
    shutdown: oneshot::Receiver<()>,
) {
    let repos = Arc::new(domain::Repositories::default());
    let storage: Arc<dyn domain::FileStorage> = Arc::new(domain::InMemoryStorage::default());
    let ctx = Arc::new(AppContext::new(config.clone(), repos, storage));
    let wiki_backend = api::routes::wiki::WikiBackend::from_config(&config)
        .await
        .expect("failed to initialize Wiki backend");

    let address = format!("{}:{}", config.server.address, config.server.port);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("failed to bind server");
    let bound_addr = listener.local_addr().expect("local addr");
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
            }
        }
        _ = shutdown_started_rx => {
            if tokio::time::timeout(SHUTDOWN_TIMEOUT, &mut server).await.is_err() {
                warn!("graceful shutdown exceeded {SHUTDOWN_TIMEOUT:?}; dropping active connections");
            }
        }
    }
}
