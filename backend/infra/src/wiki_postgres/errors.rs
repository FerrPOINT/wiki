//! Typed PostgreSQL errors stay in the persistence adapter; public envelopes stay shared.
use shared::AppError;

pub(super) fn database_error(error: sqlx::Error) -> AppError {
    match error
        .as_database_error()
        .and_then(|value| value.code())
        .as_deref()
    {
        Some("42501") => AppError::Forbidden,
        Some("23505") => AppError::conflict("duplicate entry"),
        Some("40001" | "40P01") => AppError::conflict("concurrent_change"),
        _ => AppError::database(error),
    }
}
