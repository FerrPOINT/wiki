//! Trusted server-side visibility. Local mappings are owner data, never request JSON.
use crate::AppError;
use pdlc_shared::space_access::{CentralIdentity, SpaceAdmission, SpaceRef};
use std::{collections::HashMap, time::Instant};
use uuid::Uuid;

/// A verified local profile and service scopes, supplied by the authentication adapter.
#[derive(Debug, Clone)]
pub struct WikiSpaceSubject {
    pub user_id: Uuid,
    pub identity: CentralIdentity,
    pub token_expiry: i64,
    pub global_role: String,
    pub may_read: bool,
    pub may_write: bool,
}

/// Product role ceilings from the same local profile snapshot. They never grant visibility.
#[derive(Debug, Clone)]
pub struct WikiSpaceMapping {
    pub local_space_id: Uuid,
    pub space: SpaceRef,
    pub may_edit: bool,
    pub may_admin: bool,
}

#[derive(Debug, Clone)]
pub struct WikiSpaceAdmission {
    subject: WikiSpaceSubject,
    admission: SpaceAdmission,
    local_spaces: HashMap<Uuid, WikiSpaceMapping>,
    deadline: Instant,
}
impl WikiSpaceAdmission {
    pub fn confirmed(
        subject: WikiSpaceSubject,
        mut admission: SpaceAdmission,
        mappings: Vec<WikiSpaceMapping>,
    ) -> Result<Self, AppError> {
        let started = Instant::now();
        let now = chrono::Utc::now();
        admission.validity = admission
            .validity
            .attenuate(subject.token_expiry)
            .ok_or(AppError::Forbidden)?;
        if subject.user_id.is_nil()
            || admission.identity != subject.identity
            || !admission.identity_active
            || !admission.valid()
            || !admission
                .validity
                .valid_at(now.timestamp(), subject.token_expiry)
        {
            return Err(AppError::Forbidden);
        }
        let mut local_spaces = HashMap::new();
        let mut seen = std::collections::HashSet::new();
        for mapping in mappings {
            if mapping.local_space_id.is_nil()
                || !mapping.space.valid()
                || seen.contains(&mapping.space)
                || local_spaces.contains_key(&mapping.local_space_id)
                || (mapping.may_admin && !mapping.may_edit)
            {
                return Err(AppError::Unavailable(
                    "invalid_space_identity_mapping".into(),
                ));
            }
            seen.insert(mapping.space.clone());
            local_spaces.insert(mapping.local_space_id, mapping);
        }
        let expires = chrono::DateTime::from_timestamp(admission.validity.valid_until, 0)
            .ok_or(AppError::Forbidden)?;
        let remaining = expires
            .signed_duration_since(now)
            .to_std()
            .map_err(|_| AppError::Forbidden)?;
        let deadline = started.checked_add(remaining).ok_or(AppError::Forbidden)?;
        Ok(Self {
            subject,
            admission,
            local_spaces,
            deadline,
        })
    }
    pub fn restrict_deadline(mut self, deadline: Instant) -> Self {
        self.deadline = self.deadline.min(deadline);
        self
    }
    pub fn is_current(&self) -> bool {
        Instant::now() < self.deadline
            && self
                .admission
                .validity
                .valid_at(chrono::Utc::now().timestamp(), i64::MAX)
    }
    pub fn require_current(&self) -> Result<(), AppError> {
        if self.is_current() {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
    pub fn matches_user(&self, user: Uuid) -> bool {
        self.is_current() && user == self.subject.user_id
    }
    pub fn allows(&self, local: Uuid, write: bool) -> bool {
        if !self.is_current() {
            return false;
        }
        self.local_spaces.get(&local).is_some_and(|mapping| {
            let space = &mapping.space;
            let now = chrono::Utc::now().timestamp();
            if write {
                mapping.may_edit
                    && self.subject.may_write
                    && self
                        .admission
                        .has_active_space(&self.subject.identity, space, now, i64::MAX)
            } else {
                self.subject.may_read
                    && self
                        .admission
                        .can_read_space(&self.subject.identity, space, now, i64::MAX)
            }
        })
    }
    /// Private working drafts retain the confirmed local editor ceiling.
    /// Reading them does not require a service write scope or grant edit rights.
    pub fn allows_draft_read(&self, local: Uuid) -> bool {
        self.allows(local, false) && self.local_spaces.get(&local).is_some_and(|m| m.may_edit)
    }
    pub fn draft_readable_ids(&self) -> Result<Vec<Uuid>, AppError> {
        self.require_current()?;
        Ok(self
            .local_spaces
            .keys()
            .copied()
            .filter(|id| self.allows_draft_read(*id))
            .collect())
    }
    /// Narrow a response against this original wall/monotonic window. Subtract
    /// from its existing deadline instead of starting a fresh clock after a hop.
    pub fn response_deadline(&self, until: i64) -> Result<(i64, Instant), AppError> {
        self.require_current()?;
        let validity = self
            .admission
            .validity
            .attenuate(until)
            .ok_or(AppError::Forbidden)?;
        let reduction = self
            .admission
            .validity
            .valid_until
            .checked_sub(validity.valid_until)
            .and_then(|value| u64::try_from(value).ok())
            .ok_or(AppError::Forbidden)?;
        let deadline = self
            .deadline
            .checked_sub(std::time::Duration::from_secs(reduction))
            .ok_or(AppError::Forbidden)?;
        if Instant::now() >= deadline {
            return Err(AppError::Forbidden);
        }
        Ok((validity.valid_until, deadline))
    }
    /// Bound identity, independent of whether this request grants read or write.
    pub fn subject_identity(&self) -> Result<&CentralIdentity, AppError> {
        self.require_current()?;
        Ok(&self.subject.identity)
    }
    /// Catalog revision of this request's confirmed local mapping; not a new grant.
    pub fn catalog_revision(&self, local: Uuid) -> Result<i64, AppError> {
        self.require_current()?;
        let mapping = self.local_spaces.get(&local).ok_or(AppError::Forbidden)?;
        self.admission
            .spaces
            .iter()
            .find(|entry| entry.space == mapping.space)
            .map(|entry| entry.revision)
            .ok_or(AppError::Forbidden)
    }
    pub fn writable_ids(&self) -> Result<Vec<Uuid>, AppError> {
        self.require_current()?;
        Ok(self
            .local_spaces
            .keys()
            .copied()
            .filter(|local| self.allows(*local, true))
            .collect())
    }
    pub fn readable_ids(&self) -> Result<Vec<Uuid>, AppError> {
        self.require_current()?;
        Ok(self
            .local_spaces
            .keys()
            .copied()
            .filter(|local| self.allows(*local, false))
            .collect())
    }
}

impl WikiSpaceAdmission {
    pub fn allows_method(&self, method: &str) -> bool {
        self.is_current()
            && if matches!(method, "GET" | "HEAD" | "OPTIONS") {
                self.subject.may_read
            } else {
                self.subject.may_write
            }
    }
    pub fn readable_space_ref(&self, local: Uuid) -> Result<SpaceRef, AppError> {
        if !self.allows(local, false) {
            return Err(AppError::Forbidden);
        }
        self.local_spaces
            .get(&local)
            .map(|m| m.space.clone())
            .ok_or(AppError::Forbidden)
    }
    /// The server builds the original actor window; caller JSON never chooses identity.
    pub fn read_delegation(
        &self,
    ) -> Result<pdlc_shared::space_access::SpaceReadDelegation, AppError> {
        if !self.allows_method("GET") {
            return Err(AppError::Forbidden);
        }
        Ok(pdlc_shared::space_access::SpaceReadDelegation {
            schema_version: 1,
            identity: self.subject.identity.clone(),
            validity: self.admission.validity,
        })
    }
    pub fn policy_revision(&self) -> Result<i64, AppError> {
        self.require_current()?;
        Ok(self.admission.policy_revision)
    }
    /// Retry binding excludes timestamps but changes when effective visibility or role changes.
    pub fn replay_binding(&self) -> Result<Vec<u8>, AppError> {
        self.require_current()?;
        let mut mapped: Vec<_> = self
            .local_spaces
            .iter()
            .map(|(local, mapping)| {
                (
                    *local,
                    mapping.space.clone(),
                    self.allows(*local, false),
                    self.allows(*local, true),
                    mapping.may_edit,
                    mapping.may_admin,
                )
            })
            .collect();
        mapped.sort_by_key(|(local, _, _, _, _, _)| *local);
        serde_json::to_vec(&(
            &self.subject.identity,
            self.admission.policy_revision,
            &self.subject.global_role,
            self.subject.may_read,
            self.subject.may_write,
            mapped,
        ))
        .map_err(|_| AppError::Unavailable("invalid_admission_binding".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdlc_shared::space_access::*;
    fn fixture() -> (WikiSpaceSubject, SpaceAdmission, Vec<WikiSpaceMapping>) {
        let identity = CentralIdentity {
            issuer: "https://auth.example.test".into(),
            subject: Uuid::new_v4().to_string(),
        };
        let a = SpaceRef {
            registry_instance_id: Uuid::new_v4(),
            space_id: Uuid::new_v4(),
        };
        let b = SpaceRef {
            space_id: Uuid::new_v4(),
            ..a.clone()
        };
        let now = chrono::Utc::now().timestamp();
        let subject = WikiSpaceSubject {
            user_id: Uuid::new_v4(),
            identity: identity.clone(),
            token_expiry: now + 60,
            global_role: "admin".into(),
            may_read: true,
            may_write: true,
        };
        let admission = SpaceAdmission {
            schema_version: 1,
            identity,
            identity_active: true,
            policy_revision: 1,
            policy: SpaceAccessPolicy::Selected {
                spaces: vec![a.clone()],
            },
            validity: AdmissionValidity::confirmed(now, now + 30, i64::MAX).unwrap(),
            spaces: vec![
                ConfirmedSpace {
                    space: a.clone(),
                    revision: 1,
                    state: SpaceState::Archived,
                },
                ConfirmedSpace {
                    space: b.clone(),
                    revision: 1,
                    state: SpaceState::Active,
                },
            ],
        };
        (
            subject,
            admission,
            vec![
                WikiSpaceMapping {
                    local_space_id: Uuid::new_v4(),
                    space: a,
                    may_edit: true,
                    may_admin: true,
                },
                WikiSpaceMapping {
                    local_space_id: Uuid::new_v4(),
                    space: b,
                    may_edit: true,
                    may_admin: true,
                },
            ],
        )
    }
    #[test]
    fn source_token_deadline_cannot_be_renewed_by_a_new_product_window() {
        let (subject, admission, mappings) = fixture();
        let scope = WikiSpaceAdmission::confirmed(subject, admission, mappings).unwrap();
        let expired = scope.clone().restrict_deadline(Instant::now());
        assert!(!expired.clone().is_current());
        assert!(
            !expired
                .restrict_deadline(Instant::now() + std::time::Duration::from_secs(600))
                .is_current()
        );
        assert!(scope.is_current());
    }
    #[test]
    fn response_window_only_narrows_the_original_monotonic_deadline() {
        let (subject, admission, mappings) = fixture();
        let until = admission.validity.valid_until;
        let scope = WikiSpaceAdmission::confirmed(subject, admission, mappings).unwrap();
        let (_, original) = scope.response_deadline(i64::MAX).unwrap();
        let (shorter, deadline) = scope.response_deadline(until - 5).unwrap();
        assert_eq!(shorter, until - 5);
        assert_eq!(
            original.duration_since(deadline),
            std::time::Duration::from_secs(5)
        );
        assert_eq!(
            scope.response_deadline(until + 10).unwrap(),
            (until, original)
        );
        assert!(
            scope
                .response_deadline(chrono::Utc::now().timestamp())
                .is_err()
        );
    }
    #[test]
    fn write_only_scope_keeps_identity_binding_without_creating_a_read_grant() {
        let (mut subject, mut admission, mappings) = fixture();
        subject.may_read = false;
        admission.spaces[0].state = SpaceState::Active;
        let local = mappings[0].local_space_id;
        let expected = subject.identity.clone();
        let scope = WikiSpaceAdmission::confirmed(subject, admission, mappings).unwrap();
        assert_eq!(scope.subject_identity().unwrap(), &expected);
        assert!(scope.allows(local, true));
        assert!(!scope.allows(local, false));
        assert!(scope.read_delegation().is_err());
        assert_eq!(scope.catalog_revision(local).unwrap(), 1);
        assert!(scope.catalog_revision(Uuid::new_v4()).is_err());
    }
    #[test]
    fn local_admin_does_not_open_hidden_space_or_borrow_its_active_write_state() {
        let (subject, admission, mappings) = fixture();
        let a = mappings[0].local_space_id;
        let b = mappings[1].local_space_id;
        let scope =
            WikiSpaceAdmission::confirmed(subject.clone(), admission.clone(), mappings.clone())
                .unwrap();
        assert!(scope.allows(a, false));
        assert!(!scope.allows(a, true));
        assert!(!scope.allows(b, false));
        assert!(!scope.allows(b, true));
        assert!(!scope.matches_user(Uuid::new_v4()));
        assert!(scope.matches_user(subject.user_id));
        let mut all = admission;
        all.policy = SpaceAccessPolicy::All {};
        let scope = WikiSpaceAdmission::confirmed(subject, all, mappings).unwrap();
        assert!(scope.allows(b, true));
    }
    #[test]
    fn replay_binding_is_stable_across_new_deadlines_but_changes_with_visibility_or_role() {
        let (subject, admission, mappings) = fixture();
        let first =
            WikiSpaceAdmission::confirmed(subject.clone(), admission.clone(), mappings.clone())
                .unwrap();
        let mut next = admission.clone();
        next.validity.valid_until -= 1;
        let renewed =
            WikiSpaceAdmission::confirmed(subject.clone(), next, mappings.clone()).unwrap();
        assert_eq!(
            first.replay_binding().unwrap(),
            renewed.replay_binding().unwrap()
        );
        let mut next = admission.clone();
        next.policy_revision += 1;
        let changed =
            WikiSpaceAdmission::confirmed(subject.clone(), next, mappings.clone()).unwrap();
        assert_ne!(
            first.replay_binding().unwrap(),
            changed.replay_binding().unwrap()
        );
        let mut reader = subject;
        reader.global_role = "user".into();
        reader.may_write = false;
        let changed = WikiSpaceAdmission::confirmed(reader, admission, mappings).unwrap();
        assert_ne!(
            first.replay_binding().unwrap(),
            changed.replay_binding().unwrap()
        );
        assert!(!changed.allows_method("PUT"));
    }
    #[test]
    fn invalid_mapping_inactive_identity_and_expired_confirmation_fail_closed() {
        let (subject, mut admission, mut mappings) = fixture();
        let mut disabled = admission.clone();
        disabled.identity_active = false;
        assert!(
            WikiSpaceAdmission::confirmed(subject.clone(), disabled, mappings.clone()).is_err()
        );
        let valid = admission.clone();
        admission.validity.observed_at -= 61;
        admission.validity.valid_until -= 61;
        assert!(
            WikiSpaceAdmission::confirmed(subject.clone(), admission, mappings.clone()).is_err()
        );
        assert!(
            WikiSpaceAdmission::confirmed(subject.clone(), valid.clone(), mappings.clone()).is_ok()
        );
        mappings[1].space = mappings[0].space.clone();
        assert!(WikiSpaceAdmission::confirmed(subject, valid, mappings).is_err());
    }
    #[test]
    fn forwarded_reader_uses_verified_subject_window_and_service_read_scope() {
        let (mut subject, admission, mappings) = fixture();
        subject.may_write = false;
        let scope =
            WikiSpaceAdmission::confirmed(subject.clone(), admission.clone(), mappings.clone())
                .unwrap();
        let wire = scope.read_delegation().unwrap();
        assert_eq!(wire.identity, subject.identity);
        assert_eq!(wire.validity, admission.validity);
        assert_eq!(scope.policy_revision().unwrap(), admission.policy_revision);
        subject.may_read = false;
        let denied = WikiSpaceAdmission::confirmed(subject, admission, mappings).unwrap();
        assert!(denied.read_delegation().is_err());
    }
}

impl WikiSpaceAdmission {
    pub fn allows_admin(&self, local: Uuid) -> bool {
        self.allows(local, true)
            && self
                .local_spaces
                .get(&local)
                .is_some_and(|mapping| mapping.may_admin)
    }
    pub fn confirmed_global_admin(&self) -> bool {
        self.is_current() && self.subject.global_role == "admin"
    }
}
