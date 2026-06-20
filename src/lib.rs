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
