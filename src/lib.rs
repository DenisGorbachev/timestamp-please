//! Enable the optional `uuid` feature for checked conversions between [`Timestamp`] and `uuid::Timestamp`, including in `no_std` builds. Conversions preserve the Unix time value and reject overflow, negative times, and loss of precision. UUID clock counters and their usable bit counts are discarded on input; output always uses `uuid::NoContext`.

#![no_std]
#![deny(clippy::arithmetic_side_effects)]
#![cfg_attr(not(test), deny(unused_crate_dependencies))]

#[cfg(feature = "std")]
extern crate std;

mod types;
pub use types::*;

#[cfg(feature = "serde")]
pub mod as_string;
#[cfg(feature = "serde")]
pub use as_string::*;

#[cfg(feature = "uuid")]
mod constants;
#[cfg(feature = "uuid")]
pub use constants::*;

#[cfg(feature = "uuid")]
mod functions;
#[cfg(feature = "uuid")]
pub use functions::*;
