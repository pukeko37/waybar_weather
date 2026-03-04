//! Closed error types for the domain layer.

use std::fmt;

/// Domain error enum representing all possible validation failures.
#[derive(Debug)]
pub enum WeatherError {
    /// A numeric value fell outside its valid range.
    OutOfRange {
        value: String,
        min: String,
        max: String,
        unit: &'static str,
    },
    /// Wind gusts were less than sustained wind speed.
    InvalidGusts {
        sustained: u32,
        gusts: u32,
    },
    /// Builder was missing a required field.
    MissingField(&'static str),
    /// An invalid compass direction string was provided.
    InvalidDirection(String),
    /// A time string could not be parsed.
    InvalidTime(String),
    /// A timestamp could not be parsed or converted.
    InvalidTimestamp(String),
}

impl fmt::Display for WeatherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange {
                value,
                min,
                max,
                unit,
            } => write!(
                f,
                "Value {} {} is outside valid range ({} to {})",
                value, unit, min, max
            ),
            Self::InvalidGusts { sustained, gusts } => write!(
                f,
                "Wind gusts {} km/h cannot be less than sustained wind {} km/h",
                gusts, sustained
            ),
            Self::MissingField(field) => write!(f, "{} is required", field),
            Self::InvalidDirection(dir) => write!(f, "Invalid compass direction: {}", dir),
            Self::InvalidTime(time) => write!(f, "Unable to parse time: {}", time),
            Self::InvalidTimestamp(ts) => write!(f, "Invalid timestamp: {}", ts),
        }
    }
}

impl std::error::Error for WeatherError {}
