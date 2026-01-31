# Concepts for `timestamp-please`

## `timestamp-please` package

A Rust package that contains a single `timestamp-please` crate.

## `timestamp-please` crate

A Rust crate that exports a generic `Timestamp` type that represents the count of time units since Unix epoch.

Requirements:

* Must provide conversions to and from [foreign timekeeping types](#foreign-timekeeping-type)
  * Every conversion must either:
    * Be an infallible conversion that fully preserves the underlying data (not truncate, not clamp, not `unwrap`, not return a default value)
    * Be a fallible conversion that fully preserves the underlying data if it's possible or returns an error if it's impossible
* Must support integer primitive storage types starting from 32 bits:
  * `u32`, `i32`
  * `u64`, `i64`
  * `u128`, `i128`
* Must not support floating point storage types

## Storage types

* Every storage type has a min and max value
  * Some physical values can only be represented with a combination of storage type and a power constant
* `u128` has the highest max value among the primitive storage types

### Value ranges

The min and max values are rounded down.

| Type | Power | Min (years) | Max (years) |
|------|-------|-------------|:------------|
| u32  | 0     | 0           | 136         |

## Foreign timekeeping crate

A crate that provides timekeeping-related types.

Examples:

* `core`
* `std`
* `time`
* `chrono`

Notes:

* This crate integrates only with the foreign timekeeping crates mentioned above

## Foreign timekeeping type

A type from a foreign timekeeping crate.

Examples:

* `core::time::Duration`
* `std::time::SystemTime`
* `time::OffsetDateTime`
* `chrono::DateTime<chrono::Utc>`
* `chrono::NaiveDateTime`

Notes:

* The constructors of foreign types accept only specific storage types
  * Examples
    * `core::time::Duration::from_nanos` accepts only `u64`

## Foreign timekeeping crate interop module

A Rust module that implements conversions between the types from [this crate](#timestamp-please-crate) and the time-related types from a [foreign timekeeping crate](#foreign-timekeeping-crate).

Requirements:

* Must be feature-gated
* Must `use` the items from the foreign crate
* Must contain a `use super::*;` declaration

## std interop module
