pub mod config;
pub mod error;
pub mod id;
pub mod wiki_contract;

pub use config::*;
pub use error::*;
pub use id::*;
pub use wiki_contract::*;

use chrono::{DateTime, FixedOffset, Utc};

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

pub type Timestamp = DateTime<FixedOffset>;

pub fn now() -> Timestamp {
    Utc::now().into()
}
