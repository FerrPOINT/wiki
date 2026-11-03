use std::sync::Arc;

use server::run;
use shared::AppConfig;

#[tokio::main]
async fn main() {
    pdlc_telemetry::init_tracing_with_options(
        "wiki",
        pdlc_telemetry::TracingOptions {
            json: true,
            target: true,
            current_span: true,
            default_filter: "info",
        },
    );

    let config = Arc::new(AppConfig::from_env().expect("failed to load config"));
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if !arguments.is_empty() {
        if arguments.len() != 4
            || arguments[0] != "prepare-reader"
            || arguments[2] != "--token-file"
        {
            eprintln!(
                "usage: wiki-server prepare-reader <subject-uuid> --token-file <private-file>"
            );
            std::process::exit(2);
        }
        let result = async {
            let token = std::fs::read_to_string(&arguments[3]).map_err(|_| ())?;
            if token.len() > 4096 || token.trim().is_empty() {
                return Err(());
            }
            let result = infra::wiki_postgres::prepare_registered_reader(
                &config,
                token.trim(),
                &arguments[1],
            )
            .await;
            result.map_err(|_| ())
        }
        .await;
        match result {
            Ok(id) => println!("{id}"),
            Err(()) => {
                eprintln!(
                    "reader preparation failed; verify identity, registration, schema and private configuration"
                );
                std::process::exit(1);
            }
        }
        return;
    }
    let (ready_tx, _ready_rx) = tokio::sync::oneshot::channel();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    if let Err(err) = run(config, ready_tx, shutdown_rx).await {
        tracing::error!(error = %err, "server stopped");
        std::process::exit(1);
    }
}
