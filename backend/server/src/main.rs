use std::sync::Arc;

use server::run;
use shared::AppConfig;

#[derive(Debug, PartialEq, Eq)]
enum Command<'a> {
    Serve,
    Migrate,
    PrepareReader {
        subject: &'a str,
        token_file: &'a str,
    },
}

fn parse_command(arguments: &[String]) -> Result<Command<'_>, &'static str> {
    match arguments {
        [] => Ok(Command::Serve),
        [command] if command == "migrate" => Ok(Command::Migrate),
        [command, subject, flag, token_file]
            if command == "prepare-reader" && flag == "--token-file" =>
        {
            Ok(Command::PrepareReader {
                subject,
                token_file,
            })
        }
        _ => Err("usage: wiki-server [migrate | prepare-reader <subject-uuid> --token-file <private-file>]"),
    }
}

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

    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let command = parse_command(&arguments).unwrap_or_else(|usage| {
        eprintln!("{usage}");
        std::process::exit(2);
    });
    let config = Arc::new(AppConfig::from_env().expect("failed to load config"));
    if command == Command::Migrate {
        if let Err(error) = infra::migrate_postgres_wiki_database(&config).await {
            tracing::error!(error = %error, "database migration failed");
            std::process::exit(1);
        }
        return;
    }
    if let Command::PrepareReader { subject, token_file } = command {
        let result = async {
            let token = std::fs::read_to_string(token_file).map_err(|_| ())?;
            if token.len() > 4096 || token.trim().is_empty() {
                return Err(());
            }
            let result = infra::wiki_postgres::prepare_registered_reader(
                &config,
                token.trim(),
                subject,
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

#[cfg(test)]
mod tests {
    use super::{Command, parse_command};

    fn arguments(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn no_arguments_preserve_serve() {
        assert_eq!(parse_command(&[]), Ok(Command::Serve));
    }

    #[test]
    fn migrate_and_prepare_reader_select_only_their_command() {
        let migrate = arguments(&["migrate"]);
        assert_eq!(parse_command(&migrate), Ok(Command::Migrate));
        let reader = arguments(&[
            "prepare-reader",
            "original-subject",
            "--token-file",
            "/private/token",
        ]);
        assert_eq!(
            parse_command(&reader),
            Ok(Command::PrepareReader {
                subject: "original-subject",
                token_file: "/private/token",
            })
        );
    }

    #[test]
    fn unknown_or_extra_arguments_are_rejected() {
        for values in [
            vec!["serve"],
            vec!["unknown"],
            vec!["migrate", "extra"],
            vec!["prepare-reader"],
            vec!["prepare-reader", "subject", "--token", "file"],
            vec!["prepare-reader", "subject", "--token-file", "file", "extra"],
        ] {
            assert!(parse_command(&arguments(&values)).is_err(), "{values:?}");
        }
    }
}
