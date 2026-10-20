//! Trusted owner credential. No serde and no credential-bearing Debug output.
use crate::AppError;
use pdlc_shared::space_access::{AdmissionValidity, CentralIdentity};
pub use pdlc_shared::space_lifecycle::{
    DesiredSpaceState, SpaceOwnerCommandV2, SpaceOwnerReadbackV2,
};
use std::sync::Arc;
#[derive(Clone)]
pub struct WikiSpaceOwnerAuthority {
    proof: pdlc_shared::space_lifecycle::SpaceOwnerAuthority,
    token: Arc<str>,
}
impl std::fmt::Debug for WikiSpaceOwnerAuthority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WikiSpaceOwnerAuthority")
            .field("proof", &self.proof)
            .finish_non_exhaustive()
    }
}
impl WikiSpaceOwnerAuthority {
    /// Only the registered-machine authentication adapter constructs this proof.
    pub fn confirmed(
        identity: CentralIdentity,
        token: String,
        validity: AdmissionValidity,
    ) -> Result<Self, AppError> {
        if token.is_empty()
            || token.len() > 8192
            || token
                .bytes()
                .any(|b| b.is_ascii_whitespace() || b.is_ascii_control())
        {
            return Err(AppError::Forbidden);
        }
        let proof =
            pdlc_shared::space_lifecycle::SpaceOwnerAuthority::confirmed(identity, validity)
                .ok_or(AppError::Forbidden)?;
        Ok(Self {
            proof,
            token: token.into(),
        })
    }
    pub fn require_current(&self) -> Result<(), AppError> {
        if self.proof.is_current() {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }

    pub fn token(&self) -> &str {
        &self.token
    }
    pub fn identity(&self) -> &CentralIdentity {
        self.proof.identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owner_proof_clone_keeps_deadline_and_debug_redacts_credential() {
        let now = chrono::Utc::now().timestamp();
        let validity = AdmissionValidity::confirmed(now, now + 60, i64::MAX).unwrap();
        let original = WikiSpaceOwnerAuthority::confirmed(
            CentralIdentity {
                issuer: "https://auth.example.test".into(),
                subject: "registered-owner".into(),
            },
            "synthetic-private-owner-credential".into(),
            validity,
        )
        .unwrap();
        let cloned = original.clone();
        assert_eq!(cloned.proof.valid_until(), original.proof.valid_until());
        assert!(!format!("{original:?}").contains("synthetic-private-owner-credential"));
        assert!(cloned.require_current().is_ok());
        assert!(original.require_current().is_ok());
    }
}
