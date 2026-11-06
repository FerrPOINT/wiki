//! Dedicated read processors use their own identity and existing space memberships.
//! No human session, owner credential, write role or implicit profile is minted.
use pdlc_auth_core::service_bridge::{RegisteredSpaceReader, VerifiedCentralToken};
use shared::AppError;
use uuid::Uuid;

pub fn authorize(actor: &VerifiedCentralToken) -> Result<(), AppError> {
    if super::namespace::registered_owner(&actor.context.user_id) || actor.context.scopes.len() != 1
    {
        return Err(AppError::Forbidden);
    }
    RegisteredSpaceReader::verified_machine(
        actor.clone(),
        "wiki",
        &std::env::var("WIKI_READERS__SUBJECTS").unwrap_or_default(),
    )
    .map(|_| ())
    .map_err(|_| AppError::Forbidden)
}

tokio::task_local! {
    static READER: (Uuid, VerifiedCentralToken);
}

pub async fn with_request<F: std::future::Future>(
    user: Uuid,
    actor: VerifiedCentralToken,
    future: F,
) -> Result<F::Output, AppError> {
    authorize(&actor)?;
    if user.is_nil() {
        return Err(AppError::Forbidden);
    }
    Ok(READER.scope((user, actor), future).await)
}

/// An expired or mismatched original reader is an error, never a human fallback.
pub(super) fn current(user: Uuid) -> Result<bool, AppError> {
    READER
        .try_with(|(bound, actor)| {
            if user != *bound || !actor.lifetime.is_current() {
                return Err(AppError::Forbidden);
            }
            Ok(true)
        })
        .unwrap_or(Ok(false))
}
