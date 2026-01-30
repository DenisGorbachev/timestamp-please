#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::borrow::{Borrow, BorrowMut};
use core::fmt;
use core::ops::{Deref, DerefMut};
use core::time::Duration;

/// Fixed-point Unix timestamp: `value * 10^POWER` seconds since Unix epoch.
///
/// - `Value`: integer-like storage (e.g. `u64`)
/// - `POWER`: base-10 exponent (e.g. `-3` for milliseconds)
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct Timestamp<Value = u64, const POWER: i32 = 0> {
    value: Value,
}

// TODO: Fix to display a fixed point (e.g. 1000000.123)
impl<V: fmt::Display, const POWER: i32> fmt::Display for Timestamp<V, POWER> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}e{}", self.value, POWER)
    }
}

impl<V, const POWER: i32> Timestamp<V, POWER> {
    #[inline]
    pub const fn new(value: V) -> Self {
        Self {
            value,
        }
    }
}

impl<V, const POWER: i32> From<V> for Timestamp<V, POWER> {
    #[inline]
    fn from(value: V) -> Self {
        Self::new(value)
    }
}

// TODO: Why does compiler output an error for this impl?
// impl<V, const POWER: i32> Into<V> for Timestamp<V, POWER> {
//     #[inline]
//     fn into(self) -> V {
//         self.value
//     }
// }

impl<V, const POWER: i32> Deref for Timestamp<V, POWER> {
    type Target = V;

    fn deref(&self) -> &V {
        &self.value
    }
}

impl<V, const POWER: i32> DerefMut for Timestamp<V, POWER> {
    fn deref_mut(&mut self) -> &mut V {
        &mut self.value
    }
}

impl<V, const POWER: i32> AsRef<V> for Timestamp<V, POWER> {
    fn as_ref(&self) -> &V {
        &self.value
    }
}

impl<V, const POWER: i32> Borrow<V> for Timestamp<V, POWER> {
    fn borrow(&self) -> &V {
        &self.value
    }
}

impl<V, const POWER: i32> BorrowMut<V> for Timestamp<V, POWER> {
    fn borrow_mut(&mut self) -> &mut V {
        &mut self.value
    }
}

pub type TimestampMs = Timestamp<u64, -3>;
// TODO: Add TimestampNs

// TODO: Implement conversions between Timestamp<V, POWER_IN> and Timestamp<V, POWER_OUT>
// TODO: For error handling, use `errgonomic = { version = "0.5" }` (enable in Cargo.toml)

// TODO: Rewrite the impls to generic Timestamp, not TimestampMs

// TODO: Remove this impl block
impl TimestampMs {
    pub fn as_unix_millis(&self) -> u64 {
        todo!()
    }
}

impl From<Duration> for TimestampMs {
    #[inline]
    fn from(duration: Duration) -> Self {
        let ms_u128 = duration.as_millis();
        let ms_u64 = if ms_u128 > u64::MAX as u128 { u64::MAX } else { ms_u128 as u64 };
        TimestampMs::new(ms_u64)
    }
}

impl From<TimestampMs> for Duration {
    #[inline]
    fn from(_t: TimestampMs) -> Self {
        todo!()
    }
}

#[cfg(feature = "std")]
mod interop_std {
    use super::TimestampMs;

    impl From<std::time::SystemTime> for TimestampMs {
        #[inline]
        fn from(st: std::time::SystemTime) -> Self {
            match st.duration_since(std::time::UNIX_EPOCH) {
                Ok(d) => TimestampMs::from(d),
                Err(_) => TimestampMs::new(0),
            }
        }
    }

    impl From<TimestampMs> for std::time::SystemTime {
        #[inline]
        fn from(_t: TimestampMs) -> Self {
            todo!()
        }
    }
}

// -------------------------- time crate interop (optional) --------------------------

#[cfg(feature = "time")]
mod interop_time {
    use super::TimestampMs;

    impl From<time::OffsetDateTime> for TimestampMs {
        #[inline]
        fn from(dt: time::OffsetDateTime) -> Self {
            let ns: i128 = dt.unix_timestamp_nanos();
            if ns <= 0 {
                return TimestampMs::new(0);
            }
            let ms: i128 = ns / 1_000_000;
            let ms_u64 = if ms >= u64::MAX as i128 { u64::MAX } else { ms as u64 };
            TimestampMs::new(ms_u64)
        }
    }

    impl TryFrom<TimestampMs> for time::OffsetDateTime {
        type Error = time::error::ComponentRange;

        #[inline]
        fn try_from(_t: TimestampMs) -> Result<Self, Self::Error> {
            todo!()
        }
    }
}

// -------------------------- chrono crate interop (optional) --------------------------

#[cfg(feature = "chrono")]
mod interop_chrono {
    use super::TimestampMs;

    impl From<chrono::DateTime<chrono::Utc>> for TimestampMs {
        #[inline]
        fn from(dt: chrono::DateTime<chrono::Utc>) -> Self {
            let ms: i64 = dt.timestamp_millis();
            if ms <= 0 {
                return TimestampMs::new(0);
            }
            let ms_u64 = if ms as u128 >= u64::MAX as u128 { u64::MAX } else { ms as u64 };
            TimestampMs::new(ms_u64)
        }
    }

    impl TryFrom<TimestampMs> for chrono::DateTime<chrono::Utc> {
        type Error = ();

        #[inline]
        fn try_from(t: TimestampMs) -> Result<Self, Self::Error> {
            let ms_u64 = t.as_unix_millis();
            let ms_i64 = i64::try_from(ms_u64).map_err(|_| ())?;
            chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ms_i64).ok_or(())
        }
    }
}
