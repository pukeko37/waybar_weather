//! Display module for formatting weather data as Waybar JSON output.
pub mod waybar;
pub use waybar::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::*;
    use crate::domain::{
        Astronomy as DomainAstronomy, Humidity, LastUpdated, Location, Pressure, Temperature,
        WeatherCondition, WeatherTime, WindDirection, WindSpeed,
    };

    #[test]
    fn test_waybar_output_creation() {
        let weather_data = create_mock_weather_data();
        let output = WaybarFormatter::new().format(&weather_data).unwrap();

        // Test display text format
        assert!(output.text.contains("☀️"));
        assert!(output.text.contains("20°C"));
        assert!(output.text.contains("Wellington"));

        // Test tooltip content
        assert!(output.tooltip.contains("Location: Wellington"));
        assert!(output.tooltip.contains("Temperature: 20°C"));
        assert!(output.tooltip.contains("Condition: Clear"));
        assert!(output.tooltip.contains("Humidity: 60%"));
        // Wind speed 15 km/h is Calm category (white #FFFFFF), only number colored
        assert!(output
            .tooltip
            .contains("<span foreground=\"#FFFFFF\">15</span> km/h NW"));
        assert!(output.tooltip.contains("Pressure: 1013 hPa"));
    }

    #[test]
    fn test_waybar_output_with_astronomy() {
        let weather_data = create_mock_weather_data_with_astronomy();
        let output = WaybarFormatter::new().format(&weather_data).unwrap();

        assert!(output.tooltip.contains("Sunrise: 06:30"));
        assert!(output.tooltip.contains("Solar Noon: 12:30"));
        assert!(output.tooltip.contains("Sunset: 18:30"));
        assert!(output.tooltip.contains("Daylength: 12:00"));
    }

    #[test]
    fn test_waybar_output_with_hourly_forecast() {
        let weather_data = create_mock_weather_data_with_hourly();
        let output = WaybarFormatter::new().format(&weather_data).unwrap();

        assert!(output.tooltip.contains("Upcoming Hours"));
        assert!(output.tooltip.contains("• 12:00 - 22°C Sunny"));
        // Wind speed 10 km/h is Calm (white), gusts 18 km/h is Calm (white)
        assert!(output
            .tooltip
            .contains("<span foreground=\"#FFFFFF\">10</span> km/h (Gusts: <span foreground=\"#FFFFFF\">18</span> km/h) N"));
    }

    #[test]
    fn test_error_output_formatting() {
        let error_output =
            WaybarFormatter::create_error_output("Wellington", anyhow::anyhow!("Network error"));

        assert!(error_output.text.contains("Weather unavailable"));
        assert!(error_output
            .tooltip
            .contains("Unable to fetch weather data for Wellington"));
        assert!(error_output.tooltip.contains("Network error"));
    }

    #[test]
    fn test_dew_point_display() {
        let weather_data = create_mock_weather_data();
        let output = WaybarFormatter::new().format(&weather_data).unwrap();

        // Test that dew point is calculated and displayed correctly
        // Mock data has 20°C temp and 60% humidity, so dew point should be 12°C
        assert!(output.tooltip.contains("Dew Point: 12°C"));
    }

    #[test]
    fn test_weather_api_timestamp_display() {
        let weather_data = create_mock_weather_data();
        let output = WaybarFormatter::new().format(&weather_data).unwrap();

        // Test that WeatherAPI's last updated timestamp is displayed with UTC timezone
        // Mock data uses epoch 1673620200 which corresponds to "2023-01-13 14:30Z"
        assert!(output.tooltip.contains("🕐 Updated: 2023-01-13 14:30Z"));

        // Ensure we're showing the full date-time format with timezone, not just time
        assert!(output.tooltip.contains("2023-01-13 14:30Z"));

        // Ensure we're NOT showing just time format (HH:MM:SS without date)
        let lines_with_updated: Vec<&str> = output
            .tooltip
            .lines()
            .filter(|line| line.contains("🕐 Updated:"))
            .collect();
        assert_eq!(lines_with_updated.len(), 1);
        assert!(lines_with_updated[0].contains("2023-01-13 14:30Z"));
    }

    fn create_mock_weather_data() -> WeatherData {
        let current = CurrentWeather {
            last_updated: LastUpdated::from_epoch(1673620200).unwrap(),
            temperature: Temperature::new(20).unwrap(),
            feels_like: Temperature::new(22).unwrap(),
            condition: WeatherCondition::new("Clear".to_string()),
            humidity: Humidity::new(60.0).unwrap(),
            wind_speed: WindSpeed::new(15).unwrap(),
            wind_direction: WindDirection::from_compass("NW").unwrap(),
            pressure: Pressure::new(1013).unwrap(),
        };

        let location = Location::new("Wellington".to_string());

        WeatherData {
            current,
            location,
            weather_day: None,
        }
    }

    fn create_mock_weather_data_with_astronomy() -> WeatherData {
        let mut weather_data = create_mock_weather_data();

        let astronomy = DomainAstronomy::new(
            WeatherTime::parse("06:30").unwrap(),
            WeatherTime::parse("18:30").unwrap(),
        );

        weather_data.weather_day = Some(WeatherDay {
            astronomy: Some(astronomy),
            hourly_weather: vec![],
        });

        weather_data
    }

    fn create_mock_weather_data_with_hourly() -> WeatherData {
        let mut weather_data = create_mock_weather_data();

        let hourly = HourlyWeather {
            time: WeatherTime::parse("12:00").unwrap(),
            temperature: Temperature::new(22).unwrap(),
            condition: WeatherCondition::new("Sunny".to_string()),
            wind_speed: WindSpeed::builder()
                .sustained(10)
                .with_gusts(18)
                .build()
                .unwrap(),
            wind_direction: WindDirection::from_compass("N").unwrap(),
        };

        weather_data.weather_day = Some(WeatherDay {
            astronomy: None,
            hourly_weather: vec![hourly],
        });

        weather_data
    }
}
