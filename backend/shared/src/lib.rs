#[cfg(feature = "runtime")]
pub mod config;
pub mod id;
pub mod managed_links;
pub mod wiki_contract;
pub mod wiki_space_access;

#[cfg(feature = "runtime")]
pub use config::*;
pub use id::*;
pub use wiki_contract::*;

// Fleet-shared error lives in pdlc-shared (services-base): same structured
// envelope {"error": {"code", "message"}} this service already emitted.
pub use pdlc_shared::resource_context;
pub use pdlc_shared::{AppError, AppResult, ErrorBody, ErrorEnvelope};

use chrono::{DateTime, FixedOffset, Utc};

#[cfg(all(test, feature = "runtime"))]
#[path = "lib_tests.rs"]
mod tests;

pub type Timestamp = DateTime<FixedOffset>;

pub fn now() -> Timestamp {
    Utc::now().into()
}

pub mod space_owner;
