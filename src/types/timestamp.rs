use core::borrow::{Borrow, BorrowMut};
use core::fmt;
use core::ops::{Deref, DerefMut};
use core::time::Duration;

use errgonomic::{handle, handle_opt};

const NANOS_PER_SECOND: u128 = 1_000_000_000;
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

// TODO: Rewrite it as "render" method in impl Timestamp
// TODO: Implement a very simple Display that just delegates to value
impl<V: itoa::Integer, const POWER: i32> fmt::Display for Timestamp<V, POWER> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = itoa::Buffer::new();
        let raw = buffer.format(self.value);
        let (sign, digits) = raw.strip_prefix('-').map_or(("", raw), |rest| ("-", rest));
        let power = i64::from(POWER);

        if power == 0 {
            f.write_str(sign)?;
            return f.write_str(digits);
        }

        if power > 0 {
            f.write_str(sign)?;
            f.write_str(digits)?;
            return write_zeros(f, power as usize);
        }

        let scale = (-power) as usize;
        f.write_str(sign)?;

        if digits.len() > scale {
            let split = digits.len() - scale;
            let (int_part, frac_part) = digits.split_at(split);
            f.write_str(int_part)?;
            f.write_str(".")?;
            return f.write_str(frac_part);
        }

        f.write_str("0.")?;
        write_zeros(f, scale.saturating_sub(digits.len()))?;
        f.write_str(digits)
    }
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

pub type TimestampSeconds = Timestamp<u64, 0>;
pub type TimestampMilliseconds = Timestamp<u128, -3>;
pub type TimestampMicroseconds = Timestamp<u128, -6>;
pub type TimestampNanoseconds = Timestamp<u128, -9>;

impl<V, const POWER: i32> Timestamp<V, POWER>
where
    V: Into<u128> + TryFrom<u128, Error = core::num::TryFromIntError>,
{
    #[inline]
    pub fn try_scale<const POWER_OUT: i32>(self) -> Result<Timestamp<V, POWER_OUT>, TimestampTryScaleError> {
        use TimestampTryScaleError::*;
        let value_u128: u128 = self.value.into();
        let diff = i64::from(POWER) - i64::from(POWER_OUT);
        let scaled = handle_opt!(
            scale_u128(value_u128, diff),
            ScaleFailed,
            value: value_u128,
            power_in: POWER,
            power_out: POWER_OUT
        );
        let value_out = handle!(
            V::try_from(scaled),
            TryFromFailed,
            value: scaled,
            power_out: POWER_OUT
        );
        Ok(Timestamp::new(value_out))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TimestampTryScaleError {
    ScaleFailed { value: u128, power_in: i32, power_out: i32 },
    TryFromFailed { source: core::num::TryFromIntError, value: u128, power_out: i32 },
}

// TODO: Use thiserror instead of custom Display impl
impl fmt::Display for TimestampTryScaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TimestampTryScaleError::*;
        match self {
            ScaleFailed {
                value,
                power_in,
                power_out,
            } => write!(f, "failed to scale timestamp value {value} from power {power_in} to {power_out}"),
            TryFromFailed {
                value,
                power_out,
                ..
            } => write!(f, "scaled timestamp value {value} does not fit target power {power_out}"),
        }
    }
}

// TODO: Use thiserror instead of custom Error impl
impl core::error::Error for TimestampTryScaleError {}

impl From<Duration> for Timestamp<u64, 0> {
    #[inline]
    fn from(duration: Duration) -> Self {
        Self::new(duration.as_secs())
    }
}

impl From<Duration> for Timestamp<u128, -3> {
    #[inline]
    fn from(duration: Duration) -> Self {
        Self::new(duration.as_millis())
    }
}

impl From<Duration> for Timestamp<u128, -6> {
    #[inline]
    fn from(duration: Duration) -> Self {
        Self::new(duration.as_micros())
    }
}

impl From<Duration> for Timestamp<u128, -9> {
    #[inline]
    fn from(duration: Duration) -> Self {
        Self::new(duration.as_nanos())
    }
}

impl<const POWER: i32> From<Timestamp<u64, POWER>> for Duration {
    #[inline]
    fn from(timestamp: Timestamp<u64, POWER>) -> Self {
        let value_u128 = u128::from(timestamp.value);
        let total_ns = timestamp_value_to_nanoseconds(value_u128, POWER).unwrap_or(u128::MAX);
        nanoseconds_to_duration(total_ns)
    }
}

#[inline]
#[doc(hidden)]
pub fn write_zeros(f: &mut fmt::Formatter<'_>, count: usize) -> fmt::Result {
    core::iter::repeat_n("0", count).try_for_each(|zero| f.write_str(zero))
}

#[inline]
pub fn pow10_u128(exp: u32) -> Option<u128> {
    if u64::from(exp) > MAX_POW10_U128 {
        return None;
    }

    core::iter::repeat_n(10u128, exp as usize).try_fold(1u128, |acc, value| acc.checked_mul(value))
}

#[inline]
pub fn scale_u128(value: u128, diff: i64) -> Option<u128> {
    if diff == 0 {
        return Some(value);
    }

    if diff > 0 {
        let exp = diff as u64;
        if exp > MAX_POW10_U128 {
            return if value == 0 { Some(0) } else { None };
        }
        let factor = pow10_u128(exp as u32)?;
        return value.checked_mul(factor);
    }

    let exp = diff.unsigned_abs();
    if exp > MAX_POW10_U128 {
        return Some(0);
    }
    let factor = pow10_u128(exp as u32)?;
    Some(value / factor)
}

#[inline]
pub fn clamp_u128_to_u64(value: u128) -> u64 {
    if value > u64::MAX as u128 { u64::MAX } else { value as u64 }
}

#[inline]
pub fn nanoseconds_to_duration(total_ns: u128) -> Duration {
    let secs = total_ns / NANOS_PER_SECOND;
    let nanos = (total_ns % NANOS_PER_SECOND) as u32;
    if secs > u64::MAX as u128 {
        return Duration::MAX;
    }
    Duration::new(secs as u64, nanos)
}

#[inline]
pub fn timestamp_value_to_nanoseconds(value: u128, power: i32) -> Option<u128> {
    scale_u128(value, i64::from(power) + 9)
}

#[inline]
pub fn nanoseconds_to_timestamp_value(total_ns: u128, power: i32) -> Option<u128> {
    scale_u128(total_ns, -9 - i64::from(power))
}

#[cfg(feature = "std")]
mod interop_std {
    use super::{Duration, Timestamp};

    // TODO: use `system_time.duration_since(UNIX_EPOCH)`
    // TODO: `impl TryFrom<SystemTime> for TimestampSeconds`
    // TODO: `impl TryFrom<SystemTime> for TimestampMilliseconds`
    // TODO: `impl TryFrom<SystemTime> for TimestampMicroseconds`
    // TODO: `impl TryFrom<SystemTime> for TimestampNanoseconds`

    impl<const POWER: i32> From<Timestamp<u64, POWER>> for std::time::SystemTime {
        #[inline]
        fn from(timestamp: Timestamp<u64, POWER>) -> Self {
            let duration = Duration::from(timestamp);
            let base = std::time::UNIX_EPOCH;
            base.checked_add(duration)
                .or_else(|| base.checked_add(Duration::MAX))
                .unwrap_or(base)
        }
    }
}

#[cfg(feature = "time")]
mod interop_time {
    use super::{Timestamp, clamp_u128_to_u64, nanoseconds_to_timestamp_value, timestamp_value_to_nanoseconds};
    use errgonomic::{handle, handle_bool, handle_opt};

    impl<const POWER: i32> From<time::OffsetDateTime> for Timestamp<u64, POWER> {
        #[inline]
        fn from(dt: time::OffsetDateTime) -> Self {
            let nanos: i128 = dt.unix_timestamp_nanos();
            if nanos <= 0 {
                return Timestamp::new(0);
            }
            let nanos_u128 = nanos as u128;
            let value_u128 = nanoseconds_to_timestamp_value(nanos_u128, POWER).unwrap_or(u128::MAX);
            Timestamp::new(clamp_u128_to_u64(value_u128))
        }
    }

    impl<const POWER: i32> TryFrom<Timestamp<u64, POWER>> for time::OffsetDateTime {
        type Error = ConvertTimestampToOffsetDateTimeError;

        #[inline]
        fn try_from(timestamp: Timestamp<u64, POWER>) -> Result<Self, Self::Error> {
            use ConvertTimestampToOffsetDateTimeError::*;
            let value_u128 = u128::from(timestamp.value);
            let nanos_u128 = handle_opt!(
                timestamp_value_to_nanoseconds(value_u128, POWER),
                ScaleFailed,
                value: value_u128,
                power: POWER
            );
            handle_bool!(nanos_u128 > i128::MAX as u128, NanosecondsInvalid, nanos: nanos_u128);
            let nanos_i128 = nanos_u128 as i128;
            let datetime = handle!(
                time::OffsetDateTime::from_unix_timestamp_nanos(nanos_i128),
                FromUnixTimestampNanosFailed,
                nanos: nanos_i128
            );
            Ok(datetime)
        }
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum ConvertTimestampToOffsetDateTimeError {
        ScaleFailed { value: u128, power: i32 },
        NanosecondsInvalid { nanos: u128 },
        FromUnixTimestampNanosFailed { source: time::error::ComponentRange, nanos: i128 },
    }

    impl core::fmt::Display for ConvertTimestampToOffsetDateTimeError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            use ConvertTimestampToOffsetDateTimeError::*;
            match self {
                ScaleFailed {
                    value,
                    power,
                } => write!(f, "failed to scale timestamp value {value} with power {power} to nanoseconds"),
                NanosecondsInvalid {
                    nanos,
                } => write!(f, "nanosecond value {nanos} is out of range for OffsetDateTime"),
                FromUnixTimestampNanosFailed {
                    nanos,
                    ..
                } => write!(f, "failed to build OffsetDateTime from {nanos} nanoseconds"),
            }
        }
    }

    impl core::error::Error for ConvertTimestampToOffsetDateTimeError {}
}

#[cfg(feature = "chrono")]
mod interop_chrono {
    use super::{Timestamp, clamp_u128_to_u64, nanoseconds_to_timestamp_value, timestamp_value_to_nanoseconds};
    use errgonomic::{handle_bool, handle_opt};

    impl<const POWER: i32> From<chrono::DateTime<chrono::Utc>> for Timestamp<u64, POWER> {
        #[inline]
        fn from(dt: chrono::DateTime<chrono::Utc>) -> Self {
            let nanos_opt = dt.timestamp_nanos_opt();
            let nanos = match nanos_opt {
                Some(value) => value,
                None => {
                    if dt.timestamp() <= 0 {
                        return Timestamp::new(0);
                    }
                    return Timestamp::new(u64::MAX);
                }
            };
            if nanos <= 0 {
                return Timestamp::new(0);
            }
            let nanos_u128 = nanos as u128;
            let value_u128 = nanoseconds_to_timestamp_value(nanos_u128, POWER).unwrap_or(u128::MAX);
            Timestamp::new(clamp_u128_to_u64(value_u128))
        }
    }

    impl<const POWER: i32> TryFrom<Timestamp<u64, POWER>> for chrono::DateTime<chrono::Utc> {
        type Error = ConvertTimestampToDateTimeError;

        #[inline]
        fn try_from(timestamp: Timestamp<u64, POWER>) -> Result<Self, Self::Error> {
            use ConvertTimestampToDateTimeError::*;
            let value_u128 = u128::from(timestamp.value);
            let nanos_u128 = handle_opt!(
                timestamp_value_to_nanoseconds(value_u128, POWER),
                ScaleFailed,
                value: value_u128,
                power: POWER
            );
            handle_bool!(nanos_u128 > i64::MAX as u128, NanosecondsInvalid, nanos: nanos_u128);
            let nanos_i64 = nanos_u128 as i64;
            Ok(chrono::DateTime::<chrono::Utc>::from_timestamp_nanos(nanos_i64))
        }
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum ConvertTimestampToDateTimeError {
        ScaleFailed { value: u128, power: i32 },
        NanosecondsInvalid { nanos: u128 },
    }

    impl core::fmt::Display for ConvertTimestampToDateTimeError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            use ConvertTimestampToDateTimeError::*;
            match self {
                ScaleFailed {
                    value,
                    power,
                } => write!(f, "failed to scale timestamp value {value} with power {power} to nanoseconds"),
                NanosecondsInvalid {
                    nanos,
                } => write!(f, "nanosecond value {nanos} is out of range for DateTime"),
            }
        }
    }

    impl core::error::Error for ConvertTimestampToDateTimeError {}
}

#[cfg(feature = "time")]
pub use interop_time::*;

#[cfg(feature = "chrono")]
pub use interop_chrono::*;
