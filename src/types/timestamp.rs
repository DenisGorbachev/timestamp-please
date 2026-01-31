use core::borrow::{Borrow, BorrowMut};
use core::fmt;
use core::ops::{Deref, DerefMut};
use core::time::Duration;

const MAX_POW10_U128: u64 = 38;

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

impl<V, const POWER: i32> Timestamp<V, POWER> {
    #[inline]
    pub const fn new(value: V) -> Self {
        Self {
            value,
        }
    }

    #[inline]
    pub fn into_value(self) -> V {
        self.value
    }

    #[inline]
    pub fn format_as_seconds(&self, f: &mut impl fmt::Write) -> fmt::Result
    where
        V: itoa::Integer,
    {
        let mut buffer = itoa::Buffer::new();
        let raw = buffer.format(self.value);
        let (sign, digits) = raw.strip_prefix('-').map_or(("", raw), |rest| ("-", rest));
        let power = i64::from(POWER);

        if power == 0 {
            return f.write_str(sign).and_then(|_| f.write_str(digits));
        }

        if power > 0 {
            return f
                .write_str(sign)
                .and_then(|_| f.write_str(digits))
                .and_then(|_| write_zeros(f, power as usize));
        }

        let scale = (-power) as usize;
        f.write_str(sign)?;

        if digits.len() > scale {
            let split = digits.len() - scale;
            let (int_part, frac_part) = digits.split_at(split);
            return f
                .write_str(int_part)
                .and_then(|_| f.write_str("."))
                .and_then(|_| f.write_str(frac_part));
        }

        f.write_str("0.")
            .and_then(|_| write_zeros(f, scale.saturating_sub(digits.len())))
            .and_then(|_| f.write_str(digits))
    }
}

impl<V: fmt::Display, const POWER: i32> fmt::Display for Timestamp<V, POWER> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<V, const POWER: i32> From<V> for Timestamp<V, POWER> {
    #[inline]
    fn from(value: V) -> Self {
        Self {
            value,
        }
    }
}

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

pub const UNO: i32 = 0;
pub const MILLI: i32 = -3;
pub const MICRO: i32 = -6;
pub const NANO: i32 = -9;

impl From<Duration> for Timestamp<u64, UNO> {
    #[inline]
    fn from(duration: Duration) -> Self {
        Self::new(duration.as_secs())
    }
}

impl From<Duration> for Timestamp<u128, UNO> {
    #[inline]
    fn from(duration: Duration) -> Self {
        Self::new(duration.as_secs() as u128)
    }
}

impl From<Duration> for Timestamp<u128, MILLI> {
    #[inline]
    fn from(duration: Duration) -> Self {
        Self::new(duration.as_millis())
    }
}

impl From<Duration> for Timestamp<u128, MICRO> {
    #[inline]
    fn from(duration: Duration) -> Self {
        Self::new(duration.as_micros())
    }
}

impl From<Duration> for Timestamp<u128, NANO> {
    #[inline]
    fn from(duration: Duration) -> Self {
        Self::new(duration.as_nanos())
    }
}

impl From<Timestamp<u64, NANO>> for Duration {
    #[inline]
    fn from(timestamp: Timestamp<u64, NANO>) -> Self {
        Duration::from_nanos(timestamp.value)
    }
}

// `impl From<Timestamp<u128, NANO>> for Duration` is not implementable because `Duration::from_nanos` accepts only `u64`

#[inline]
#[doc(hidden)]
pub fn write_zeros(f: &mut impl fmt::Write, count: usize) -> fmt::Result {
    core::iter::repeat_n("0", count).try_for_each(|zero| f.write_str(zero))
}

#[inline]
pub fn pow10_u128(exp: u32) -> Option<u128> {
    if u64::from(exp) > MAX_POW10_U128 {
        return None;
    }

    core::iter::repeat_n(10u128, exp as usize).try_fold(1u128, |acc, value| acc.checked_mul(value))
}

#[cfg(feature = "std")]
mod interop_std {
    use super::*;

    macro_rules! impl_try_from_system_time {
        ($target:ty) => {
            impl TryFrom<std::time::SystemTime> for $target {
                type Error = std::time::SystemTimeError;

                #[inline]
                fn try_from(system_time: std::time::SystemTime) -> Result<Self, Self::Error> {
                    let duration = system_time.duration_since(std::time::UNIX_EPOCH)?;
                    Ok(Self::from(duration))
                }
            }
        };
    }

    impl_try_from_system_time!(Timestamp<u64, UNO>);
    impl_try_from_system_time!(Timestamp<u128, MILLI>);
    impl_try_from_system_time!(Timestamp<u128, MICRO>);
    impl_try_from_system_time!(Timestamp<u128, NANO>);
}

#[cfg(feature = "time")]
mod interop_time {
    use super::*;
    use time::OffsetDateTime;
    use time::error::ComponentRange;

    impl From<OffsetDateTime> for Timestamp<i128, NANO> {
        #[inline]
        fn from(dt: OffsetDateTime) -> Self {
            Timestamp::new(dt.unix_timestamp_nanos())
        }
    }

    impl TryFrom<Timestamp<i128, NANO>> for OffsetDateTime {
        type Error = ComponentRange;

        #[inline]
        fn try_from(timestamp: Timestamp<i128, NANO>) -> Result<Self, Self::Error> {
            OffsetDateTime::from_unix_timestamp_nanos(timestamp.value)
        }
    }
}

#[cfg(feature = "chrono")]
mod interop_chrono {
    use super::*;
    use chrono::{DateTime, TimeZone, Utc};

    impl<Tz: TimeZone> TryFrom<DateTime<Tz>> for Timestamp<i128, NANO> {
        type Error = UnrepresentableChronoDateTimeError;

        #[inline]
        fn try_from(dt: DateTime<Tz>) -> Result<Self, Self::Error> {
            dt.timestamp_nanos_opt()
                .map(i128::from)
                .map(Self::new)
                .ok_or(UnrepresentableChronoDateTimeError)
        }
    }

    impl TryFrom<Timestamp<i128, NANO>> for DateTime<Utc> {
        type Error = UnrepresentableChronoDateTimeError;

        #[inline]
        fn try_from(timestamp: Timestamp<i128, NANO>) -> Result<Self, Self::Error> {
            let nanos: i64 = timestamp
                .value
                .try_into()
                .map_err(|_| UnrepresentableChronoDateTimeError)?;
            Ok(Self::from_timestamp_nanos(nanos))
        }
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct UnrepresentableChronoDateTimeError;

    impl fmt::Display for UnrepresentableChronoDateTimeError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("chrono timestamp is out of range for nanosecond precision")
        }
    }

    impl core::error::Error for UnrepresentableChronoDateTimeError {}
}

#[cfg(feature = "chrono")]
pub use interop_chrono::*;
