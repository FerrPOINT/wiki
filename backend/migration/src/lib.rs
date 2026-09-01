use std::path::{Path, PathBuf};

pub async fn migrator() -> Result<sqlx::migrate::Migrator, sqlx::migrate::MigrateError> {
    sqlx::migrate::Migrator::new(migrations_dir()).await
}

pub fn migrations_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("WIKI_MIGRATIONS_DIR") {
        return PathBuf::from(path);
    }

    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("migrations")
}
