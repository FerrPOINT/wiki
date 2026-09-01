use anyhow::{Context, Result, bail};
use sqlx::{Row, postgres::PgPoolOptions};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
struct AppliedMigration {
    description: String,
    success: bool,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("migration failed: {error:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let url = database_url()?;
    let cmd = std::env::args().nth(1).unwrap_or_else(|| "up".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .context("failed to connect to database")?;

    match cmd.as_str() {
        "up" => {
            migration::MIGRATOR
                .run(&pool)
                .await
                .context("migration up failed")?;
            println!("SQLx migrations applied");
        }
        "status" => {
            print_status(&pool).await?;
        }
        "fresh" => {
            reset_public_schema(&pool).await?;
            migration::MIGRATOR
                .run(&pool)
                .await
                .context("migration fresh failed")?;
            println!("database recreated, SQLx migrations applied");
        }
        "down" => {
            let applied = applied_migrations(&pool).await?;
            let Some(current_version) = applied.keys().next_back().copied() else {
                println!("no migrations applied");
                return Ok(());
            };
            let target = applied
                .keys()
                .rev()
                .find(|version| **version < current_version)
                .copied()
                .unwrap_or(0);
            migration::MIGRATOR
                .undo(&pool, target)
                .await
                .context("migration down failed")?;
            println!("reverted migration {current_version}");
        }
        other => bail!("unknown command: {other} (use up|status|fresh|down)"),
    }

    Ok(())
}

fn database_url() -> Result<String> {
    std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("WIKI_DATABASE__URL"))
        .context("DATABASE_URL or WIKI_DATABASE__URL must be set")
}

async fn print_status(pool: &sqlx::PgPool) -> Result<()> {
    let applied = applied_migrations(pool).await?;
    let known_versions = migration::MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .map(|migration| migration.version)
        .collect::<BTreeSet<_>>();
    let pending = known_versions
        .iter()
        .filter(|version| !applied.contains_key(version))
        .count();

    println!(
        "known {}, applied {}, pending {} SQLx migrations:",
        known_versions.len(),
        applied.len(),
        pending
    );

    for migration in migration::MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
    {
        let state = match applied.get(&migration.version) {
            Some(applied) if applied.success => "applied",
            Some(_) => "failed",
            None => "pending",
        };
        println!(
            "  {state:8} {} {}",
            migration.version, migration.description
        );
    }

    for (version, applied) in applied
        .iter()
        .filter(|(version, _)| !known_versions.contains(version))
    {
        let state = if applied.success {
            "database-only"
        } else {
            "failed-db-only"
        };
        println!("  {state:14} {version} {}", applied.description);
    }

    Ok(())
}

async fn applied_migrations(pool: &sqlx::PgPool) -> Result<BTreeMap<i64, AppliedMigration>> {
    let table_name: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('_sqlx_migrations')::text")
            .fetch_one(pool)
            .await
            .context("failed to inspect SQLx migrations table")?;

    if table_name.is_none() {
        return Ok(BTreeMap::new());
    }

    let rows = sqlx::query(
        r#"
        SELECT version, description, success
        FROM _sqlx_migrations
        ORDER BY version
        "#,
    )
    .fetch_all(pool)
    .await
    .context("failed to read SQLx migrations table")?;

    let mut applied = BTreeMap::new();
    for row in rows {
        applied.insert(
            row.try_get("version")?,
            AppliedMigration {
                description: row.try_get("description")?,
                success: row.try_get("success")?,
            },
        );
    }
    Ok(applied)
}

async fn reset_public_schema(pool: &sqlx::PgPool) -> Result<()> {
    sqlx::query("DROP SCHEMA IF EXISTS public CASCADE")
        .execute(pool)
        .await
        .context("failed to drop public schema")?;
    sqlx::query("CREATE SCHEMA public")
        .execute(pool)
        .await
        .context("failed to recreate public schema")?;
    sqlx::query("GRANT ALL ON SCHEMA public TO public")
        .execute(pool)
        .await
        .context("failed to restore public schema grants")?;
    Ok(())
}
