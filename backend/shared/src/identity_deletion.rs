//! Transport authorization belongs to the private API machine factory. This
//! in-process deadline only attenuates that verified authority across storage I/O.
pub use pdlc_shared::identity_lifecycle::{
    DeletionRequest, OwnerFreezeReceipt, OwnerReferenceSummary, OwnerReleaseReceipt,
};
use std::time::Instant;
#[derive(Clone)]
pub struct IdentityDeletionWindow {
    deadline: Instant,
    valid_until: i64,
}

impl IdentityDeletionWindow {
    pub fn restricted(deadline: Instant, valid_until: i64) -> Self {
        Self {
            deadline,
            valid_until,
        }
    }
    pub fn is_current(&self) -> bool {
        Instant::now() < self.deadline && chrono::Utc::now().timestamp() < self.valid_until
    }
    pub fn require_current(&self) -> Result<(), crate::AppError> {
        if self.is_current() {
            Ok(())
        } else {
            Err(crate::AppError::Unauthorized)
        }
    }
    /// Freshness only: retain both original clocks without manufacturing human authority.
    pub fn commit_valid_until(
        &self,
        database_observed_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<chrono::DateTime<chrono::Utc>, crate::AppError> {
        let remaining = self
            .deadline
            .checked_duration_since(Instant::now())
            .ok_or(crate::AppError::Unauthorized)?;
        // DB anchor was read before this remaining duration, so network delay
        // shortens the fixed window even when client and database clocks differ.
        let monotonic_cap = database_observed_at
            .checked_add_signed(
                chrono::Duration::from_std(remaining).map_err(|_| crate::AppError::Unauthorized)?,
            )
            .ok_or(crate::AppError::Unauthorized)?;
        let until = chrono::DateTime::from_timestamp(self.valid_until, 0)
            .ok_or(crate::AppError::Unauthorized)?
            .min(monotonic_cap);
        self.require_current()?;
        Ok(until)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn database_anchor_preserves_monotonic_expiry_when_the_database_clock_is_behind() {
        let anchor = chrono::Utc::now() - chrono::Duration::hours(1);
        let window = IdentityDeletionWindow::restricted(
            Instant::now() + std::time::Duration::from_secs(10),
            chrono::Utc::now().timestamp() + 3600,
        );
        let until = window.commit_valid_until(anchor).unwrap();
        assert!(until <= anchor + chrono::Duration::seconds(10));
        assert!(until > anchor);
    }
}
