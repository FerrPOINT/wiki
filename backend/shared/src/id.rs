use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use uuid::Uuid;

macro_rules! uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            pub fn nil() -> Self {
                Self(Uuid::nil())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }
    };
}

uuid_id!(UserId);
uuid_id!(AttachmentId);
uuid_id!(AuditLogId);
uuid_id!(SpaceId);
uuid_id!(DocumentId);
uuid_id!(DocumentRevisionId);
uuid_id!(DocumentTemplateId);
uuid_id!(TaskDossierId);
uuid_id!(PhaseDossierId);
uuid_id!(EvidenceId);

#[cfg(test)]
mod tests;
