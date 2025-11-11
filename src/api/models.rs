//! API models for deserializing WeatherAPI.com JSON responses and converting to domain types.
//!
//! WeatherAPI.com provides current weather, forecast, and astronomy data through a unified API.
//! This module handles the JSON response structure and converts it to our domain models.

use crate::domain::*;
use anyhow::{Context, Result};
use serde::Deserialize;

/// Root weather API response from WeatherAPI.com
#[derive(Debug, Deserialize)]
pub struct WeatherApiResponse {
    pub location: LocationApi,
    pub current: CurrentApi,
    pub forecast: Option<ForecastApi>,
}

impl TryFrom<WeatherApiResponse> for WeatherData {
    type Error = anyhow::Error;

    fn try_from(value: WeatherApiResponse) -> Result<Self> {
        let current = value
            .current
            .try_into()
            .context("Failed to parse current conditions")?;

        let location = Location::new(value.location.name);

        // Parse location's local time for filtering using the localtime string
        let location_local_hour = value
            .location
            .localtime
            .split(' ')
            .nth(1)
            .and_then(|time_part| time_part.split(':').next())
            .and_then(|hour_str| hour_str.parse::<u32>().ok())
            .unwrap_or(0);

        let weather_day = value
            .forecast
            .and_then(|f| f.forecastday.into_iter().next())
            .map(|day| day.try_into())
            .transpose()
            .context("Failed to parse weather day data")?
            .map(|day: WeatherDay| day.filter_future_hours(location_local_hour));

        Ok(WeatherData {
            current,
            location,
            weather_day,
        })
    }
}

/// Location information from WeatherAPI.com
#[derive(Debug, Deserialize)]
pub struct LocationApi {
    pub name: String,
    pub localtime: String,
}

/// Forecast data from WeatherAPI.com
#[derive(Debug, Deserialize)]
pub struct ForecastApi {
    pub forecastday: Vec<ForecastDayApi>,
}

/// Single forecast day from WeatherAPI.com
#[derive(Debug, Deserialize)]
pub struct ForecastDayApi {
    pub astro: Option<AstroApi>,
    pub hour: Vec<HourApi>,
}

impl TryFrom<ForecastDayApi> for WeatherDay {
    type Error = anyhow::Error;

    fn try_from(value: ForecastDayApi) -> Result<Self> {
        let astronomy = value
            .astro
            .map(|ast| ast.try_into())
            .transpose()
            .context("Failed to parse astronomy data")?;

        let hourly_weather = value
            .hour
            .into_iter()
            .map(|hour| hour.try_into())
            .collect::<Result<Vec<_>>>()
            .context("Failed to parse hourly weather data")?;

        Ok(WeatherDay {
            astronomy,
            hourly_weather,
        })
    }
}

/// Astronomy data from WeatherAPI.com
#[derive(Debug, Deserialize)]
pub struct AstroApi {
    pub sunrise: String,
    pub sunset: String,
}

impl TryFrom<AstroApi> for crate::domain::Astronomy {
    type Error = anyhow::Error;

    fn try_from(value: AstroApi) -> Result<Self> {
        let sunrise = WeatherTime::parse(&value.sunrise)
            .with_context(|| format!("Failed to parse sunrise: {}", value.sunrise))?;

        let sunset = WeatherTime::parse(&value.sunset)
            .with_context(|| format!("Failed to parse sunset: {}", value.sunset))?;

        Ok(crate::domain::Astronomy::new(sunrise, sunset))
    }
}

/// Hourly weather data from WeatherAPI.com
#[derive(Debug, Deserialize)]
pub struct HourApi {
    pub time: String,
    pub temp_c: f64,
    pub condition: ConditionApi,
    pub wind_kph: f64,
    pub wind_dir: String,
    pub gust_kph: f64,
}

impl TryFrom<HourApi> for HourlyWeather {
    type Error = anyhow::Error;

    fn try_from(value: HourApi) -> Result<Self> {
        // Parse time from ISO format "2023-01-13 14:00"
        let time_parts: Vec<&str> = value.time.split(' ').collect();
        let time_str = time_parts
            .get(1)
            .ok_or_else(|| anyhow::anyhow!("Invalid time format: {}", value.time))?;

        let time = WeatherTime::parse(time_str)
            .with_context(|| format!("Failed to parse time: {}", time_str))?;

        let temperature = Temperature::new(value.temp_c.round() as i32)
            .with_context(|| format!("Temperature out of range: {}", value.temp_c))?;

        let condition = WeatherCondition::new(value.condition.text);

        let sustained_wind = value.wind_kph.round() as u32;
        let gust_wind = value.gust_kph.round() as u32;

        let wind_speed = if gust_wind > sustained_wind {
            WindSpeed::builder()
                .sustained(sustained_wind)
                .with_gusts(gust_wind)
                .build()
                .with_context(|| {
                    format!(
                        "Invalid wind data: sustained {} km/h, gusts {} km/h",
                        sustained_wind, gust_wind
                    )
                })?
        } else {
            WindSpeed::new(sustained_wind)
                .with_context(|| format!("Wind speed out of range: {}", sustained_wind))?
        };

        let wind_direction = WindDirection::from_compass(&value.wind_dir)
            .with_context(|| format!("Invalid wind direction: {}", value.wind_dir))?;

        Ok(HourlyWeather {
            time,
            temperature,
            condition,
            wind_speed,
            wind_direction,
        })
    }
}

/// Current weather data from WeatherAPI.com
#[derive(Debug, Deserialize)]
pub struct CurrentApi {
    pub last_updated_epoch: i64,
    pub last_updated: String,
    pub temp_c: f64,
    pub condition: ConditionApi,
    pub wind_kph: f64,
    pub wind_dir: String,
    pub pressure_mb: f64,
    pub humidity: i32,
    pub feelslike_c: f64,
    pub gust_kph: f64,
}

impl TryFrom<CurrentApi> for CurrentWeather {
    type Error = anyhow::Error;

    fn try_from(value: CurrentApi) -> Result<Self> {
        let last_updated = LastUpdated::from_epoch(value.last_updated_epoch)
            .or_else(|_| LastUpdated::from_api_format(&value.last_updated))
            .with_context(|| {
                format!(
                    "Failed to parse last updated timestamp: epoch={}, string={}",
                    value.last_updated_epoch, value.last_updated
                )
            })?;

        let temperature = Temperature::new(value.temp_c.round() as i32)
            .with_context(|| format!("Temperature out of range: {}", value.temp_c))?;

        let feels_like = Temperature::new(value.feelslike_c.round() as i32).with_context(|| {
            format!("Feels like temperature out of range: {}", value.feelslike_c)
        })?;

        let humidity = Humidity::new(value.humidity as f32)
            .with_context(|| format!("Humidity out of range: {}", value.humidity))?;

        let sustained_wind = value.wind_kph.round() as u32;
        let gust_wind = value.gust_kph.round() as u32;

        let wind_speed = if gust_wind > sustained_wind {
            WindSpeed::builder()
                .sustained(sustained_wind)
                .with_gusts(gust_wind)
                .build()
                .with_context(|| {
                    format!(
                        "Invalid wind data: sustained {} km/h, gusts {} km/h",
                        sustained_wind, gust_wind
                    )
                })?
        } else {
            WindSpeed::new(sustained_wind)
                .with_context(|| format!("Wind speed out of range: {}", sustained_wind))?
        };

        let pressure = Pressure::new(value.pressure_mb.round() as u32)
            .with_context(|| format!("Pressure out of range: {}", value.pressure_mb))?;

        let condition = WeatherCondition::new(value.condition.text);
        let wind_direction = WindDirection::from_compass(&value.wind_dir)
            .with_context(|| format!("Invalid wind direction: {}", value.wind_dir))?;

        Ok(CurrentWeather {
            last_updated,
            temperature,
            feels_like,
            condition,
            humidity,
            wind_speed,
            wind_direction,
            pressure,
        })
    }
}

/// Weather condition from WeatherAPI.com
#[derive(Debug, Deserialize)]
pub struct ConditionApi {
    pub text: String,
}

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
    pub astronomy: Option<crate::domain::Astronomy>,
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
