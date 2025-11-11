//! Core domain types for weather data with compile-time safety and validation.

use anyhow::{Context, Result};
use std::fmt;
use std::marker::PhantomData;
use time::{macros::format_description, OffsetDateTime, PrimitiveDateTime, Time};

// === Range Validation Trait ===

/// Trait for types that validate values within a compile-time range
pub trait RangeValidated<T>
where
    T: PartialOrd + Copy + fmt::Display,
{
    const MIN: T;
    const MAX: T;
    const UNIT: &'static str;

    /// Validate that a value is within the range [MIN, MAX]
    fn validate(value: T) -> Result<()> {
        if value < Self::MIN || value > Self::MAX {
            anyhow::bail!(
                "Value {} {} is outside valid range ({} to {})",
                value,
                Self::UNIT,
                Self::MIN,
                Self::MAX
            );
        }
        Ok(())
    }
}

// === Generic Range-Validated Type ===

/// Generic validated type with const generic range bounds
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RangeValidatedValue<T, R>
where
    T: PartialOrd + Copy + fmt::Display,
    R: RangeValidated<T>,
{
    value: T,
    _range: PhantomData<R>,
}

impl<T, R> RangeValidatedValue<T, R>
where
    T: PartialOrd + Copy + fmt::Display,
    R: RangeValidated<T>,
{
    /// Create a new validated value - the only way to construct this type
    pub fn new(value: T) -> Result<Self> {
        R::validate(value)?;
        Ok(Self {
            value,
            _range: PhantomData,
        })
    }

    /// Get the validated value
    pub fn value(&self) -> T {
        self.value
    }
}

// === Range Definitions ===

/// Weather temperature range (-40 to 55°C)
#[derive(Debug, Clone, Copy)]
pub struct WeatherTempRange;
impl RangeValidated<i32> for WeatherTempRange {
    const MIN: i32 = -40;
    const MAX: i32 = 55;
    const UNIT: &'static str = "°C";
}

/// Humidity percentage range (0 to 100%)
#[derive(Debug, Clone, Copy)]
pub struct HumidityRange;
impl RangeValidated<f32> for HumidityRange {
    const MIN: f32 = 0.0;
    const MAX: f32 = 100.0;
    const UNIT: &'static str = "%";
}

/// Atmospheric pressure range (800 to 1100 hPa)
#[derive(Debug, Clone, Copy)]
pub struct PressureRange;
impl RangeValidated<u32> for PressureRange {
    const MIN: u32 = 800;
    const MAX: u32 = 1100;
    const UNIT: &'static str = "hPa";
}

/// Wind speed range (0 to 500 km/h)
#[derive(Debug, Clone, Copy)]
pub struct WindSpeedRange;
impl RangeValidated<u32> for WindSpeedRange {
    const MIN: u32 = 0;
    const MAX: u32 = 500;
    const UNIT: &'static str = "km/h";
}

/// Temperature in Celsius with validation
pub type Temperature = RangeValidatedValue<i32, WeatherTempRange>;

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}°C", self.value())
    }
}

impl Temperature {
    /// Get temperature in Celsius
    pub fn as_celsius(&self) -> i32 {
        self.value()
    }
}

/// Humidity percentage with validation
pub type Humidity = RangeValidatedValue<f32, HumidityRange>;

impl fmt::Display for Humidity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.as_int())
    }
}

impl Humidity {
    /// Get humidity as integer percentage for display
    pub fn as_int(&self) -> i32 {
        self.value().round() as i32
    }

    /// Calculate dew point given temperature and humidity
    pub fn dew_point(&self, temperature: &Temperature) -> Temperature {
        let temp_c = temperature.as_celsius();
        let humidity_percent = self.as_int();
        let dew_point = temp_c - (100 - humidity_percent) / 5;
        Temperature::new(dew_point).unwrap_or(*temperature)
    }
}

/// Wind speed category based on sustained wind speed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindSpeedCategory {
    /// Calm winds: 0-19 km/h
    Calm,
    /// Moderate breezes: 20-50 km/h
    ModerateBreezes,
    /// Gales: 51-88 km/h
    Gales,
    /// Storms: 89-117 km/h
    Storms,
    /// Hurricane force: 118+ km/h
    Hurricane,
}

impl WindSpeedCategory {
    /// Get the color associated with this wind speed category
    pub fn color(&self) -> &'static str {
        match self {
            Self::Calm => "#FFFFFF",
            Self::ModerateBreezes => "#00AA00",
            Self::Gales => "#FFA500",
            Self::Storms => "#FF0000",
            Self::Hurricane => "#9B30FF",
        }
    }
}

/// Builder for creating WindSpeed with fluent API
pub struct WindSpeedBuilder {
    sustained: Option<u32>,
    gusts: Option<u32>,
}

impl WindSpeedBuilder {
    /// Create a new builder instance
    pub fn new() -> Self {
        Self {
            sustained: None,
            gusts: None,
        }
    }

    /// Set the sustained wind speed in km/h
    pub fn sustained(mut self, speed: u32) -> Self {
        self.sustained = Some(speed);
        self
    }

    /// Set the wind gust speed in km/h
    pub fn with_gusts(mut self, gusts: u32) -> Self {
        self.gusts = Some(gusts);
        self
    }

    /// Build the WindSpeed instance with validation
    pub fn build(self) -> anyhow::Result<WindSpeed> {
        let sustained = self
            .sustained
            .ok_or_else(|| anyhow::anyhow!("Sustained wind speed is required"))?;
        WindSpeed::with_gusts(sustained, self.gusts)
    }
}

impl Default for WindSpeedBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Wind speed with gusts and validation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindSpeed {
    sustained: u32,
    gusts: Option<u32>,
}

impl WindSpeed {
    /// Create wind speed with just sustained wind
    pub fn new(sustained: u32) -> anyhow::Result<Self> {
        Self::with_gusts(sustained, None)
    }

    /// Create wind speed with sustained wind and optional gusts
    pub fn with_gusts(sustained: u32, gusts: Option<u32>) -> anyhow::Result<Self> {
        // Validate sustained wind speed using range
        WindSpeedRange::validate(sustained)?;

        // Validate gusts if present
        if let Some(gusts) = gusts {
            WindSpeedRange::validate(gusts)?;
            if gusts < sustained {
                anyhow::bail!(
                    "Wind gusts {} km/h cannot be less than sustained wind {} km/h",
                    gusts,
                    sustained
                );
            }
        }

        Ok(Self { sustained, gusts })
    }

    /// Create a builder for fluent construction
    pub fn builder() -> WindSpeedBuilder {
        WindSpeedBuilder::new()
    }

    /// Categorize wind speed based on sustained wind
    pub fn category(&self) -> WindSpeedCategory {
        match self.sustained {
            0..=19 => WindSpeedCategory::Calm,
            20..=50 => WindSpeedCategory::ModerateBreezes,
            51..=88 => WindSpeedCategory::Gales,
            89..=117 => WindSpeedCategory::Storms,
            118.. => WindSpeedCategory::Hurricane,
        }
    }

    /// Get the color for this wind speed category
    pub fn color(&self) -> &'static str {
        self.category().color()
    }

    /// Format wind speed with Pango color markup for Waybar tooltip
    /// Only colors the numbers, not the units
    pub fn format_colored(&self) -> String {
        let sustained_color = self.color();
        let sustained_colored = format!(
            "<span foreground=\"{}\">{}</span>",
            sustained_color, self.sustained
        );

        match self.gusts {
            Some(gusts) => {
                // Create a temporary WindSpeed with gust value to get its category
                let gust_category = match gusts {
                    0..=19 => WindSpeedCategory::Calm,
                    20..=50 => WindSpeedCategory::ModerateBreezes,
                    51..=88 => WindSpeedCategory::Gales,
                    89..=117 => WindSpeedCategory::Storms,
                    118.. => WindSpeedCategory::Hurricane,
                };
                let gust_color = gust_category.color();
                let gust_colored = format!("<span foreground=\"{}\">{}</span>", gust_color, gusts);

                format!(
                    "{} km/h (Gusts: {} km/h)",
                    sustained_colored, gust_colored
                )
            }
            None => format!("{} km/h", sustained_colored),
        }
    }

    /// Format wind speed compactly for title bar display (e.g., "43 km/h")
    /// Only shows sustained wind speed, colored by category
    pub fn format_colored_compact(&self) -> String {
        let sustained_color = self.color();
        format!(
            "<span foreground=\"{}\">{}</span> km/h",
            sustained_color, self.sustained
        )
    }
}

impl fmt::Display for WindSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.gusts {
            Some(gusts) => write!(f, "{} km/h (Gusts: {} km/h)", self.sustained, gusts),
            None => write!(f, "{} km/h", self.sustained),
        }
    }
}

/// Atmospheric pressure with validation
pub type Pressure = RangeValidatedValue<u32, PressureRange>;

impl fmt::Display for Pressure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} hPa", self.value())
    }
}

/// Wind direction as compass point with validation
#[derive(Debug, Clone, PartialEq)]
pub struct WindDirection {
    direction: String,
}

impl fmt::Display for WindDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.direction)
    }
}

impl WindDirection {
    /// Create wind direction from compass string with validation
    pub fn from_compass(compass: &str) -> Result<Self> {
        let direction = compass.to_uppercase();
        const VALID_DIRECTIONS: &[&str] = &[
            "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
            "NW", "NNW",
        ];

        if VALID_DIRECTIONS.contains(&direction.as_str()) {
            Ok(Self { direction })
        } else {
            anyhow::bail!("Invalid compass direction: {}", compass)
        }
    }
}

/// Location name with fallback handling
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    name: String,
}

impl Location {
    /// Create location, using "Unknown" for empty names
    pub fn new(name: String) -> Self {
        let name = if name.trim().is_empty() {
            "Unknown".to_string()
        } else {
            name
        };
        Self { name }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Weather time with parsing and formatting
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeatherTime {
    time: Time,
}

impl WeatherTime {
    /// Parse time from various formats
    pub fn parse(time_str: &str) -> Result<Self> {
        // Try different format patterns
        let trimmed = time_str.trim();

        // Try 12-hour format with AM/PM using different hour format (e.g., "06:30 AM")
        if let Ok(time) = Time::parse(
            trimmed,
            &format_description!("[hour repr:12]:[minute] [period]"),
        ) {
            return Ok(Self { time });
        }

        // Try 24-hour format (e.g., "18:30")
        if let Ok(time) = Time::parse(trimmed, &format_description!("[hour]:[minute]")) {
            return Ok(Self { time });
        }

        // Try 12-hour without space (e.g., "6:30AM")
        if let Ok(time) = Time::parse(
            trimmed,
            &format_description!("[hour repr:12]:[minute][period]"),
        ) {
            return Ok(Self { time });
        }

        // Try with optional leading zero for 12-hour format
        if let Ok(time) = Time::parse(
            trimmed,
            &format_description!("[hour padding:none repr:12]:[minute] [period]"),
        ) {
            return Ok(Self { time });
        }

        anyhow::bail!("Unable to parse time: {}", time_str)
    }

    /// Get hour in 24-hour format
    pub fn hour24(self) -> u32 {
        self.time.hour() as u32
    }

    /// Get minute
    pub fn minute(self) -> u32 {
        self.time.minute() as u32
    }

    /// Format as 24-hour time
    pub fn format_24h(self) -> String {
        format!("{:02}:{:02}", self.hour24(), self.minute())
    }

    /// Get total seconds from midnight
    pub fn total_seconds(self) -> u32 {
        (self.time.hour() as u32) * 3600
            + (self.time.minute() as u32) * 60
            + (self.time.second() as u32)
    }
}

impl fmt::Display for WeatherTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_24h())
    }
}

/// Duration representing day length
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Duration {
    total_minutes: u32,
}

impl Duration {
    /// Create duration from total minutes
    pub fn from_minutes(minutes: u32) -> Self {
        Self {
            total_minutes: minutes,
        }
    }

    /// Get hours component
    pub fn hours(self) -> u32 {
        self.total_minutes / 60
    }

    /// Get minutes component (0-59)
    pub fn minutes(self) -> u32 {
        self.total_minutes % 60
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{:02}", self.hours(), self.minutes())
    }
}

/// Astronomical data with sunrise/sunset times and calculations
#[derive(Debug, Clone, PartialEq)]
pub struct Astronomy {
    sunrise: WeatherTime,
    sunset: WeatherTime,
}

impl Astronomy {
    /// Create astronomical data with sunrise and sunset times
    pub fn new(sunrise: WeatherTime, sunset: WeatherTime) -> Self {
        Self { sunrise, sunset }
    }

    /// Get sunrise time
    pub fn sunrise(&self) -> WeatherTime {
        self.sunrise
    }

    /// Get sunset time
    pub fn sunset(&self) -> WeatherTime {
        self.sunset
    }

    /// Calculate day length between sunrise and sunset
    pub fn day_length(&self) -> Duration {
        let sunrise_seconds = self.sunrise.total_seconds();
        let sunset_seconds = self.sunset.total_seconds();

        let day_length_seconds = if sunset_seconds >= sunrise_seconds {
            sunset_seconds - sunrise_seconds
        } else {
            // Handle case where sunset is next day (shouldn't happen normally)
            (24 * 3600 - sunrise_seconds) + sunset_seconds
        };

        Duration::from_minutes(day_length_seconds / 60)
    }

    /// Calculate solar noon as midpoint between sunrise and sunset
    pub fn solar_noon(&self) -> Result<WeatherTime> {
        let sunrise_seconds = self.sunrise.total_seconds();
        let sunset_seconds = self.sunset.total_seconds();

        let solar_noon_seconds = if sunset_seconds >= sunrise_seconds {
            sunrise_seconds + (sunset_seconds - sunrise_seconds) / 2
        } else {
            // Handle case where sunset is next day
            let day_length = (24 * 3600 - sunrise_seconds) + sunset_seconds;
            (sunrise_seconds + day_length / 2) % (24 * 3600)
        };

        let solar_noon_hours = solar_noon_seconds / 3600;
        let solar_noon_minutes = (solar_noon_seconds % 3600) / 60;

        WeatherTime::parse(&format!(
            "{:02}:{:02}",
            solar_noon_hours, solar_noon_minutes
        ))
    }
}

/// Weather condition with icon mapping
#[derive(Debug, Clone, PartialEq)]
pub struct WeatherCondition {
    description: String,
}

impl WeatherCondition {
    /// Create weather condition
    pub fn new(description: String) -> Self {
        Self { description }
    }

    /// Get appropriate weather icon
    pub fn icon(&self) -> &'static str {
        let condition_lower = self.description.to_lowercase();
        match condition_lower.as_str() {
            c if c.contains("sunny") || c.contains("clear") => "☀️",
            c if c.contains("partly") || c.contains("partial") => "⛅",
            c if c.contains("cloudy") || c.contains("overcast") => "☁️",
            c if c.contains("rain") || c.contains("drizzle") => "🌧️",
            c if c.contains("storm") || c.contains("thunder") => "⛈️",
            c if c.contains("snow") || c.contains("blizzard") => "🌨️",
            c if c.contains("fog") || c.contains("mist") => "🌫️",
            c if c.contains("wind") => "💨",
            _ => "🌤️",
        }
    }
}

impl fmt::Display for WeatherCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

/// Timestamp representing when weather data was last updated by the API
#[derive(Debug, Clone, PartialEq)]
pub struct LastUpdated {
    datetime: OffsetDateTime,
}

impl LastUpdated {
    /// Create from Unix timestamp (epoch seconds)
    pub fn from_epoch(epoch_seconds: i64) -> Result<Self> {
        let datetime = OffsetDateTime::from_unix_timestamp(epoch_seconds)
            .map_err(|e| anyhow::anyhow!("Invalid timestamp {}: {}", epoch_seconds, e))?;
        Ok(Self { datetime })
    }

    /// Create from WeatherAPI format string (e.g., "2023-01-13 14:30")
    pub fn from_api_format(api_string: &str) -> Result<Self> {
        let format = format_description!("[year]-[month]-[day] [hour]:[minute]");
        let primitive_datetime = PrimitiveDateTime::parse(api_string, &format)
            .with_context(|| format!("Failed to parse API timestamp: {}", api_string))?;

        // Assume UTC timezone
        let datetime = primitive_datetime.assume_utc();
        Ok(Self { datetime })
    }

    /// Format as yyyy-MM-dd HH:mmZ for display (Z indicates UTC, ISO 8601 standard)
    pub fn format_display(&self) -> String {
        let format = format_description!("[year]-[month]-[day] [hour]:[minute]Z");
        self.datetime
            .format(&format)
            .unwrap_or_else(|_| "Invalid date".to_string())
    }
}

impl fmt::Display for LastUpdated {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_display())
    }
}
