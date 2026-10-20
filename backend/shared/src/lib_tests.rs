use axum::{body::to_bytes, response::IntoResponse};
use http::StatusCode;
use serde_json::Value;

use crate::{AppError, Timestamp, now};

#[test]
fn error_constructors() {
    let e = AppError::not_found("issue", "TT-15");
    assert!(matches!(e, AppError::NotFound(_)));
    assert_eq!(e.to_string(), "not found: issue TT-15 not found");

    let e = AppError::invalid_input("bad request");
    assert!(matches!(e, AppError::InvalidInput(_)));

    let e = AppError::conflict("duplicate");
    assert!(matches!(e, AppError::Conflict(_)));

    let e = AppError::database("pg timeout");
    assert!(matches!(e, AppError::Database(_)));

    let duplicate =
        AppError::database("duplicate key value violates unique constraint \"uq_example\"");
    assert!(matches!(duplicate, AppError::Conflict(message) if message == "duplicate entry"));

    let e = AppError::internal("boom");
    assert!(matches!(e, AppError::Internal(_)));
}

#[tokio::test]
async fn app_error_response_uses_structured_public_envelope() {
    let response = AppError::invalid_input("space key is required").into_response();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(json["error"]["message"], "space key is required");
}

#[tokio::test]
async fn app_error_response_masks_internal_messages() {
    let response = AppError::database("connection string leaked").into_response();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["error"]["code"], "INTERNAL_ERROR");
    assert_eq!(json["error"]["message"], "internal server error");
}

#[test]
fn now_returns_timestamp() {
    let t: Timestamp = now();
    assert!(t.timestamp() > 1_700_000_000);
}

#[tokio::test]
async fn app_error_response_keeps_guard_status_without_private_diagnostics() {
    for (error, expected, code) in [
        (AppError::Forbidden, StatusCode::FORBIDDEN, "FORBIDDEN"),
        (
            AppError::conflict("concurrent_change"),
            StatusCode::CONFLICT,
            "CONFLICT",
        ),
        (
            AppError::Database("private fixture detail".into()),
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
        ),
    ] {
        let response = error.into_response();
        assert_eq!(response.status(), expected);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let envelope: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(envelope["error"]["code"], code);
        assert!(!String::from_utf8_lossy(&body).contains("private fixture"));
    }
}
