//! Platform-independent time support.
//!
//! Reexports either `std::time` or `web_time`, depending on platform.

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::{Instant, SystemTime, SystemTimeError, UNIX_EPOCH};
#[cfg(target_arch = "wasm32")]
pub use web_time::{Instant, SystemTime, SystemTimeError, UNIX_EPOCH};
