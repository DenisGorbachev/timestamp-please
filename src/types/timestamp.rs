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

    /// Returns the underlying storage value.
    ///
    /// A generic `Into<V>` implementation would conflict with `core`'s blanket `Into` implementation, while the corresponding generic `From<Timestamp<V, POWER>> for V` implementation is prohibited by the orphan rule.
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
            let scale = match usize::try_from(power) {
                Ok(scale) => scale,
                Err(_) => return Err(fmt::Error),
            };
            return f
                .write_str(sign)
                .and_then(|_| f.write_str(digits))
                .and_then(|_| write_zeros(f, scale));
        }

        let scale = match power
            .checked_neg()
            .and_then(|power| usize::try_from(power).ok())
        {
            Some(scale) => scale,
            None => return Err(fmt::Error),
        };
        f.write_str(sign)?;

        if digits.len() > scale {
            let split = match digits.len().checked_sub(scale) {
                Some(split) => split,
                None => return Err(fmt::Error),
            };
            let (int_part, frac_part) = digits.split_at(split);
            return f
                .write_str(int_part)
                .and_then(|_| f.write_str("."))
                .and_then(|_| f.write_str(frac_part));
        }

        let zero_count = match scale.checked_sub(digits.len()) {
            Some(zero_count) => zero_count,
            None => return Err(fmt::Error),
        };
        f.write_str("0.")
            .and_then(|_| write_zeros(f, zero_count))
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

pub type TimestampMs = Timestamp<u128, MILLI>;
pub type TimestampNs = Timestamp<u128, NANO>;

impl From<Duration> for Timestamp<u64, UNO> {
    /// PRUNING: converts the duration to whole seconds, discarding the subsecond remainder because this timestamp cannot represent it.
    #[inline]
    fn from(duration: Duration) -> Self {
        Self::new(duration.as_secs())
    }
}

impl From<Duration> for Timestamp<u128, UNO> {
    /// PRUNING: converts the duration to whole seconds, discarding the subsecond remainder because this timestamp cannot represent it.
    #[inline]
    fn from(duration: Duration) -> Self {
        Self::new(duration.as_secs() as u128)
    }
}

impl From<Duration> for Timestamp<u128, MILLI> {
    /// PRUNING: converts the duration to whole milliseconds, discarding the sub-millisecond remainder because this timestamp cannot represent it.
    #[inline]
    fn from(duration: Duration) -> Self {
        Self::new(duration.as_millis())
    }
}

impl From<Duration> for Timestamp<u128, MICRO> {
    /// PRUNING: converts the duration to whole microseconds, discarding the sub-microsecond remainder because this timestamp cannot represent it.
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
    use core::num::TryFromIntError;
    use errgonomic::handle;
    use std::time::*;
    use thiserror::Error;

    macro_rules! impl_from_system_time {
        ($(#[$meta:meta])* $target:ty) => {
            impl From<SystemTime> for $target {
                $(#[$meta])*
                #[inline]
                fn from(system_time: SystemTime) -> Self {
                    let duration = system_time
                        .duration_since(UNIX_EPOCH)
                        .expect("always succeeds because UNIX_EPOCH is the minimum possible value");
                    Self::from(duration)
                }
            }
        };
    }

    macro_rules! impl_now {
        ($(#[$meta:meta])* $target:ty) => {
            impl $target {
                $(#[$meta])*
                pub fn now() -> Self {
                    Self::from(SystemTime::now())
                }
            }
        };
    }

    macro_rules! impl_all {
        ($(#[$meta:meta])* $target:ty) => {
            impl_from_system_time!($(#[$meta])* $target);
            impl_now!($(#[$meta])* $target);
        };
    }

    impl_all!(
        /// PRUNING: stores time in whole seconds, discarding the subsecond remainder because this timestamp cannot represent it.
        Timestamp<u64, UNO>
    );
    impl_all!(
        /// PRUNING: stores time in whole milliseconds, discarding the sub-millisecond remainder because this timestamp cannot represent it.
        Timestamp<u128, MILLI>
    );
    impl_all!(
        /// PRUNING: stores time in whole microseconds, discarding the sub-microsecond remainder because this timestamp cannot represent it.
        Timestamp<u128, MICRO>
    );
    impl_all!(Timestamp<u128, NANO>);

    impl TryFrom<Duration> for Timestamp<u64, MILLI> {
        type Error = TimestampTryNowError;

        /// PRUNING: converts the duration to whole milliseconds, discarding the sub-millisecond remainder because this timestamp cannot represent it.
        #[inline]
        fn try_from(duration: Duration) -> Result<Self, Self::Error> {
            use TimestampTryNowError::*;

            let milliseconds = duration.as_millis();
            let value = handle!(u64::try_from(milliseconds), TryFromFailed, duration, milliseconds);
            Ok(Self::new(value))
        }
    }

    impl TryFrom<SystemTime> for Timestamp<u64, MILLI> {
        type Error = TimestampTryNowError;

        /// PRUNING: converts the system time to whole milliseconds, discarding the sub-millisecond remainder because this timestamp cannot represent it.
        #[inline]
        fn try_from(system_time: SystemTime) -> Result<Self, Self::Error> {
            use TimestampTryNowError::*;

            let duration = handle!(system_time.duration_since(UNIX_EPOCH), DurationSinceFailed, system_time);
            Self::try_from(duration)
        }
    }

    macro_rules! impl_try_now {
        ($target:ty) => {
            impl $target {
                /// PRUNING: reports the current time in whole milliseconds, discarding the sub-millisecond remainder because this timestamp cannot represent it.
                #[inline]
                pub fn try_now() -> Result<Self, TimestampTryNowError> {
                    Self::try_from(SystemTime::now())
                }
            }
        };
    }

    impl_try_now!(Timestamp<u64, MILLI>);

    #[derive(Error, Debug)]
    pub enum TimestampTryNowError {
        #[error("current system time is before the Unix epoch")]
        DurationSinceFailed { source: SystemTimeError, system_time: SystemTime },
        #[error("duration is outside the range of a `u64` millisecond timestamp: {milliseconds} milliseconds")]
        TryFromFailed { source: TryFromIntError, duration: Duration, milliseconds: u128 },
    }
}

#[cfg(feature = "std")]
pub use interop_std::*;

#[cfg(feature = "time")]
mod interop_time {
    use super::*;
    use core::num::TryFromIntError;
    use errgonomic::{handle, handle_opt};
    use thiserror::Error;
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

    impl TryFrom<OffsetDateTime> for Timestamp<u64, MILLI> {
        type Error = TimestampOffsetDateTimeConversionError;

        /// PRUNING: converts the offset date-time to whole milliseconds, discarding the sub-millisecond remainder because this timestamp cannot represent it.
        #[inline]
        fn try_from(date_time: OffsetDateTime) -> Result<Self, Self::Error> {
            use TimestampOffsetDateTimeConversionError::*;

            let seconds = handle!(u64::try_from(date_time.unix_timestamp()), TryFromFailed, date_time);
            let value = handle_opt!(
                seconds
                    .checked_mul(1_000)
                    .and_then(|milliseconds| milliseconds.checked_add(u64::from(date_time.millisecond()))),
                ValueOutOfRange,
                date_time
            );
            Ok(Self::new(value))
        }
    }

    #[derive(Error, Debug)]
    pub enum TimestampOffsetDateTimeConversionError {
        #[error("offset date-time '{date_time}' is before the Unix epoch")]
        TryFromFailed { source: TryFromIntError, date_time: OffsetDateTime },
        #[error("offset date-time '{date_time}' is outside the range of a `u64` millisecond timestamp")]
        ValueOutOfRange { date_time: OffsetDateTime },
    }
}

#[cfg(feature = "time")]
pub use interop_time::*;

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
