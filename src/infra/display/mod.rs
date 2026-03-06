//! Display module for formatting weather data as Waybar JSON output.
pub mod formatting;
pub mod waybar;
pub use waybar::*;

#[cfg(test)]
mod tests {
    use super::formatting::*;
    use super::*;
    use crate::app::WeatherFormatter;
    use crate::domain::{
        Astronomy, CurrentWeather, Humidity, HourlyWeather, LastUpdated, Location, Pressure,
        Temperature, WeatherCondition, WeatherData, WeatherDay, WeatherTime, WindDirection,
        WindSpeed, WindSpeedCategory,
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

    #[test]
    fn test_condition_icon() {
        let condition = WeatherCondition::new("Clear".to_string());
        assert_eq!(condition_icon(&condition), "☀️");

        let cloudy = WeatherCondition::new("Partly cloudy".to_string());
        assert_eq!(condition_icon(&cloudy), "⛅");

        let rainy = WeatherCondition::new("Light rain".to_string());
        assert_eq!(condition_icon(&rainy), "🌧️");
    }

    #[test]
    fn test_wind_speed_format_colored() {
        let calm = WindSpeed::new(10).unwrap();
        assert_eq!(
            format_wind_colored(&calm),
            "<span foreground=\"#FFFFFF\">10</span> km/h"
        );

        let moderate = WindSpeed::new(30).unwrap();
        assert_eq!(
            format_wind_colored(&moderate),
            "<span foreground=\"#00AA00\">30</span> km/h"
        );

        let gale = WindSpeed::new(60).unwrap();
        assert_eq!(
            format_wind_colored(&gale),
            "<span foreground=\"#FFA500\">60</span> km/h"
        );

        let storm = WindSpeed::new(100).unwrap();
        assert_eq!(
            format_wind_colored(&storm),
            "<span foreground=\"#FF0000\">100</span> km/h"
        );

        let hurricane = WindSpeed::new(150).unwrap();
        assert_eq!(
            format_wind_colored(&hurricane),
            "<span foreground=\"#9B30FF\">150</span> km/h"
        );
    }

    #[test]
    fn test_wind_speed_format_colored_with_gusts() {
        let calm_with_moderate_gusts = WindSpeed::with_gusts(15, Some(45)).unwrap();
        assert_eq!(calm_with_moderate_gusts.category(), WindSpeedCategory::Calm);
        assert_eq!(
            format_wind_colored(&calm_with_moderate_gusts),
            "<span foreground=\"#FFFFFF\">15</span> km/h (Gusts: <span foreground=\"#00AA00\">45</span> km/h)"
        );

        let moderate_with_gale_gusts = WindSpeed::with_gusts(25, Some(60)).unwrap();
        assert_eq!(moderate_with_gale_gusts.category(), WindSpeedCategory::ModerateBreezes);
        assert_eq!(
            format_wind_colored(&moderate_with_gale_gusts),
            "<span foreground=\"#00AA00\">25</span> km/h (Gusts: <span foreground=\"#FFA500\">60</span> km/h)"
        );
    }

    // === Wind Speed Color Tests (moved from domain) ===

    #[test]
    fn test_wind_speed_calm_color() {
        let calm = WindSpeed::new(10).unwrap();
        assert_eq!(format_wind_colored(&calm), "<span foreground=\"#FFFFFF\">10</span> km/h");
    }

    #[test]
    fn test_wind_speed_moderate_color() {
        let moderate = WindSpeed::new(35).unwrap();
        assert_eq!(format_wind_colored(&moderate), "<span foreground=\"#00AA00\">35</span> km/h");
    }

    #[test]
    fn test_wind_speed_gale_color() {
        let gale = WindSpeed::new(70).unwrap();
        assert_eq!(format_wind_colored(&gale), "<span foreground=\"#FFA500\">70</span> km/h");
    }

    #[test]
    fn test_wind_speed_storm_color() {
        let storm = WindSpeed::new(100).unwrap();
        assert_eq!(format_wind_colored(&storm), "<span foreground=\"#FF0000\">100</span> km/h");
    }

    #[test]
    fn test_wind_speed_hurricane_color() {
        let hurricane = WindSpeed::new(150).unwrap();
        assert_eq!(format_wind_colored(&hurricane), "<span foreground=\"#9B30FF\">150</span> km/h");
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

        let astronomy = Astronomy::new(
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
