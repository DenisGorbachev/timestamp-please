use crate::pow10_u128;
use errgonomic::{handle_bool, handle_opt};
use thiserror::Error;

/// Changes a nonnegative timestamp's decimal power without losing precision.
pub fn rescale_timestamp(value: u128, from_power: i32, to_power: i32) -> Result<u128, RescaleTimestampError> {
    use RescaleTimestampError::*;

    if value == 0 {
        return Ok(0);
    }

    let scale = pow10_u128(from_power.abs_diff(to_power));
    if from_power < to_power {
        let scale = handle_opt!(scale, PrecisionLoss, value, from_power, to_power);
        handle_bool!(!value.is_multiple_of(scale), PrecisionLoss, value, from_power, to_power);
        return Ok(handle_opt!(value.checked_div(scale), ValueOutOfRange, value, from_power, to_power));
    }

    Ok(handle_opt!(scale.and_then(|scale| value.checked_mul(scale)), ValueOutOfRange, value, from_power, to_power))
}

#[derive(Error, Copy, Clone, Debug, Eq, PartialEq)]
pub enum RescaleTimestampError {
    #[error("timestamp {value} with power {from_power} cannot be represented exactly with power {to_power}")]
    PrecisionLoss { value: u128, from_power: i32, to_power: i32 },
    #[error("timestamp {value} with power {from_power} exceeds the u128 range with power {to_power}")]
    ValueOutOfRange { value: u128, from_power: i32, to_power: i32 },
}
