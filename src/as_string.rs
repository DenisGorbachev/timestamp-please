use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Deserializer, Serializer};

use crate::Timestamp;

pub fn serialize<V, S, const POWER: i32>(timestamp: &Timestamp<V, POWER>, serializer: S) -> Result<S::Ok, S::Error>
where
    V: fmt::Display,
    S: Serializer,
{
    serializer.collect_str(timestamp)
}

pub fn deserialize<'de, V, D, const POWER: i32>(deserializer: D) -> Result<Timestamp<V, POWER>, D::Error>
where
    V: FromStr,
    V::Err: fmt::Display,
    D: Deserializer<'de>,
{
    let value = <&str>::deserialize(deserializer)?;
    value
        .parse::<V>()
        .map(Timestamp::new)
        .map_err(serde::de::Error::custom)
}
