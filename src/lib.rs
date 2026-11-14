#![no_std]
#![deny(clippy::arithmetic_side_effects)]

#[cfg(feature = "std")]
extern crate std;

mod types;
pub use types::*;

#[cfg(feature = "serde")]
pub mod as_string;
#[cfg(feature = "serde")]
pub use as_string::*;

mod constants;
pub use constants::*;

mod functions;
pub use functions::*;
