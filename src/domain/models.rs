//! Domain aggregate types for weather data.

use crate::domain::{
    Astronomy, Humidity, LastUpdated, Location, Pressure, Temperature, WeatherCondition,
    WeatherTime, WindDirection, WindSpeed,
};

/// Domain model for complete weather data
#[derive(Debug)]
pub struct WeatherData {
    pub current: CurrentWeather,
    pub location: Location,
    pub weather_day: Option<WeatherDay>,
}

/// Domain model for current weather conditions
#[derive(Debug)]
pub struct CurrentWeather {
    pub last_updated: LastUpdated,
    pub temperature: Temperature,
    pub feels_like: Temperature,
    pub condition: WeatherCondition,
    pub humidity: Humidity,
    pub wind_speed: WindSpeed,
    pub wind_direction: WindDirection,
    pub pressure: Pressure,
}

/// Domain model for weather day with astronomy and hourly data
#[derive(Debug)]
pub struct WeatherDay {
    pub astronomy: Option<Astronomy>,
    pub hourly_weather: Vec<HourlyWeather>,
}

impl WeatherDay {
    /// Filter hourly weather to only include future hours using location's local time
    pub fn filter_future_hours(mut self, current_local_hour: u32) -> Self {
        // Filter to keep only future hours (including current hour for some tolerance)
        self.hourly_weather
            .retain(|hourly| hourly.time.hour24() >= current_local_hour);

        // Calculate the end hour: either 12 hours from now or 23:00, whichever is smaller
        // Subtract 1 to make it exclusive (12 hours max, not 13)
        let max_end_hour = std::cmp::min(current_local_hour + 11, 23);

        // Keep only hours up to the calculated end hour
        self.hourly_weather
            .retain(|hourly| hourly.time.hour24() <= max_end_hour);

        self
    }
}

/// Domain model for hourly weather
#[derive(Debug)]
pub struct HourlyWeather {
    pub time: WeatherTime,
    pub temperature: Temperature,
    pub condition: WeatherCondition,
    pub wind_speed: WindSpeed,
    pub wind_direction: WindDirection,
}
