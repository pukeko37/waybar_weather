//! Waybar output formatter for weather data with functional composition.

use crate::api::models::WeatherData;

use anyhow::Result;

use serde::Serialize;

/// Waybar JSON output format
#[derive(Debug, Serialize)]
pub struct WaybarOutput {
    pub text: String,
    pub tooltip: String,
}

/// Formatter for creating Waybar JSON output from weather data
pub struct WaybarFormatter;

impl WaybarFormatter {
    /// Create a new Waybar formatter
    pub fn new() -> Self {
        Self
    }

    /// Format weather data into Waybar output
    pub fn format(&self, weather_data: &WeatherData) -> Result<WaybarOutput> {
        let text = self.format_display_text(weather_data);
        let tooltip = self.format_tooltip(weather_data)?;

        Ok(WaybarOutput { text, tooltip })
    }

    /// Create error output for display when weather data is unavailable
    pub fn create_error_output(location: &str, error: anyhow::Error) -> WaybarOutput {
        let text = "🌤️ -- Weather unavailable".to_string();
        let tooltip = format!(
            "Unable to fetch weather data for {}\n\
             \n\
             Error: {}\n\
             Service: WeatherAPI.com\n\
             \n\
             Last attempt: {}",
            location,
            error,
            time::OffsetDateTime::now_utc()
                .format(&time::macros::format_description!(
                    "[year]-[month]-[day] [hour]:[minute]Z"
                ))
                .unwrap_or_else(|_| "Unknown".to_string())
        );

        WaybarOutput { text, tooltip }
    }

    /// Format the main display text (icon + temperature + wind speed + location)
    fn format_display_text(&self, weather_data: &WeatherData) -> String {
        format!(
            "{} {}/ {} {}",
            weather_data.current.condition.icon(),
            weather_data.current.temperature,
            weather_data.current.wind_speed.format_colored_compact(),
            weather_data.location
        )
    }

    /// Format the detailed tooltip information
    fn format_tooltip(&self, weather_data: &WeatherData) -> Result<String> {
        let dew_point = weather_data
            .current
            .humidity
            .dew_point(&weather_data.current.temperature);

        let basic_info = format!(
            "📍 Location: {}\n\
             🌡️ Temperature: {}\n\
             🌤️ Condition: {}\n\
             🤚 Feels like: {}\n\
             💧 Humidity: {} (Dew Point: {})\n\
             💨 Wind: {} {}\n\
             📊 Pressure: {}",
            weather_data.location,
            weather_data.current.temperature,
            weather_data.current.condition,
            weather_data.current.feels_like,
            weather_data.current.humidity,
            dew_point,
            weather_data.current.wind_speed.format_colored(),
            weather_data.current.wind_direction,
            weather_data.current.pressure
        );

        let astronomy_info = weather_data
            .weather_day
            .as_ref()
            .and_then(|day| day.astronomy.as_ref())
            .map(|ast| {
                format!(
                    "\n🌅 Sunrise: {}\n\
                     🌞 Solar Noon: {}\n\
                     🌇 Sunset: {}\n\
                     ⏳ Daylength: {}",
                    ast.sunrise(),
                    ast.solar_noon().unwrap_or_else(|_| ast.sunrise()), // fallback to sunrise if solar noon calculation fails
                    ast.sunset(),
                    ast.day_length()
                )
            })
            .unwrap_or_default();

        let hourly_forecast = weather_data
            .weather_day
            .as_ref()
            .filter(|day| !day.hourly_weather.is_empty())
            .map(|day| {
                let forecast = day
                    .hourly_weather
                    .iter()
                    .map(|hour| self.format_hourly_entry(hour))
                    .collect::<Vec<_>>()
                    .join("\n");

                format!("\n\n⏰ Upcoming Hours:\n{}", forecast)
            })
            .unwrap_or_default();

        let update_info = format!("\n\n🕐 Updated: {}", weather_data.current.last_updated);

        Ok(format!(
            "{}{}{}{}",
            basic_info, astronomy_info, hourly_forecast, update_info
        ))
    }

    /// Format a single hourly forecast entry
    fn format_hourly_entry(&self, hourly: &crate::api::models::HourlyWeather) -> String {
        format!(
            "• {} - {} {}\n          Wind: {} {}",
            hourly.time,
            hourly.temperature,
            hourly.condition,
            hourly.wind_speed.format_colored(),
            hourly.wind_direction
        )
    }
}

impl Default for WaybarFormatter {
    fn default() -> Self {
        Self::new()
    }
}
