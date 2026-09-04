pub mod config;
pub mod id;
pub mod wiki_contract;

pub use config::*;
pub use id::*;
pub use wiki_contract::*;

// Fleet-shared error lives in sdlc-shared (services-base): same structured
// envelope {"error": {"code", "message"}} this service already emitted.
pub use sdlc_shared::{AppError, AppResult, ErrorBody, ErrorEnvelope};

use chrono::{DateTime, FixedOffset, Utc};

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

pub type Timestamp = DateTime<FixedOffset>;

pub fn now() -> Timestamp {
    Utc::now().into()
}
