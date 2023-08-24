use std::fmt;

use chrono::NaiveTime;
use serde::{
    de::{self, Unexpected, Visitor},
    Deserializer,
};

struct TimeVisitor {}

impl<'de> Visitor<'de> for TimeVisitor {
    type Value = NaiveTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a time string in the format HH:MM")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        NaiveTime::parse_from_str(v, "%H:%M")
            .map_err(|_| de::Error::invalid_value(Unexpected::Str(v), &self))
    }
}

pub fn deserialize_time<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(TimeVisitor {})
}
