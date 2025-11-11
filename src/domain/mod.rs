//! Domain value objects for weather data with type-level safety and validation.

pub mod types;

pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_creation_and_conversion() {
        let temp = Temperature::new(25).expect("Valid temperature");
        assert_eq!(temp.as_celsius(), 25);
        assert_eq!(temp.to_string(), "25°C");

        let temp_cold = Temperature::new(-10).expect("Valid temperature");
        assert_eq!(temp_cold.as_celsius(), -10);
    }

    #[test]
    fn test_temperature_validation() {
        // Should reject extreme temperatures
        assert!(Temperature::new(-500).is_err());
        assert!(Temperature::new(200).is_err());

        // Should accept reasonable weather temperatures
        assert!(Temperature::new(-40).is_ok());
        assert!(Temperature::new(55).is_ok());

        // Should reject temperatures outside realistic range
        assert!(Temperature::new(-41).is_err());
        assert!(Temperature::new(56).is_err());
    }

    #[test]
    fn test_humidity_validation() {
        assert!(Humidity::new(50.5).is_ok());
        assert!(Humidity::new(0.0).is_ok());
        assert!(Humidity::new(100.0).is_ok());

        // Invalid values
        assert!(Humidity::new(-1.0).is_err());
        assert!(Humidity::new(101.0).is_err());
    }

    #[test]
    fn test_wind_speed_creation() {
        let wind = WindSpeed::new(15).expect("Valid wind speed");
        assert_eq!(wind.to_string(), "15 km/h");
    }

    #[test]
    fn test_wind_speed_with_gusts() {
        let wind = WindSpeed::with_gusts(20, Some(35)).expect("Valid wind with gusts");
        assert_eq!(wind.to_string(), "20 km/h (Gusts: 35 km/h)");
    }

    #[test]
    fn test_wind_speed_gust_validation() {
        // Should reject gusts less than sustained wind
        assert!(WindSpeed::with_gusts(30, Some(25)).is_err());

        // Should accept gusts equal to sustained wind
        assert!(WindSpeed::with_gusts(25, Some(25)).is_ok());

        // Should accept gusts greater than sustained wind
        assert!(WindSpeed::with_gusts(25, Some(40)).is_ok());

        // Should reject unreasonably high values
        assert!(WindSpeed::with_gusts(600, Some(700)).is_err());
        assert!(WindSpeed::new(600).is_err());
    }

    #[test]
    fn test_wind_speed_builder_pattern() {
        let wind = WindSpeed::builder()
            .sustained(25)
            .with_gusts(40)
            .build()
            .expect("Valid wind with builder");

        assert_eq!(wind.to_string(), "25 km/h (Gusts: 40 km/h)");
    }

    #[test]
    fn test_wind_speed_builder_sustained_only() {
        let wind = WindSpeed::builder()
            .sustained(15)
            .build()
            .expect("Valid wind with builder");

        assert_eq!(wind.to_string(), "15 km/h");
    }

    #[test]
    fn test_wind_speed_builder_requires_sustained() {
        let result = WindSpeed::builder().with_gusts(30).build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Sustained wind speed is required"));
    }

    #[test]
    fn test_validated_trait_clean_interface() {
        // Test that Validated trait now has a clean, ergonomic interface
        // WindSpeed::new() takes just u32 (sustained wind) - much cleaner than tuple
        let wind = WindSpeed::new(25).expect("Valid sustained wind");
        assert_eq!(wind.to_string(), "25 km/h");

        // For more complex construction, use specific methods
        let complex_wind = WindSpeed::with_gusts(20, Some(35)).expect("Valid wind with gusts");
        assert_eq!(complex_wind.to_string(), "20 km/h (Gusts: 35 km/h)");

        // Or use the builder pattern for fluent API
        let builder_wind = WindSpeed::builder()
            .sustained(30)
            .with_gusts(45)
            .build()
            .expect("Valid wind from builder");
        assert_eq!(builder_wind.to_string(), "30 km/h (Gusts: 45 km/h)");
    }

    #[test]
    fn test_pressure_creation() {
        let pressure = Pressure::new(1013).expect("Valid pressure");
        assert_eq!(pressure.to_string(), "1013 hPa");

        // Test validation
        assert!(Pressure::new(500).is_err()); // Too low
        assert!(Pressure::new(1200).is_err()); // Too high
    }

    #[test]
    fn test_location_creation() {
        let location = Location::new("Wellington".to_string());
        assert_eq!(location.to_string(), "Wellington");

        let empty_location = Location::new("".to_string());
        assert_eq!(empty_location.to_string(), "Unknown");
    }

    #[test]
    fn test_weather_time_parsing() {
        let time = WeatherTime::parse("06:30 AM").expect("Valid time");
        assert_eq!(time.hour24(), 6);
        assert_eq!(time.minute(), 30);

        let time_pm = WeatherTime::parse("06:30 PM").expect("Valid time");
        assert_eq!(time_pm.hour24(), 18);
        assert_eq!(time_pm.minute(), 30);

        // Test 24-hour format
        let time_24h = WeatherTime::parse("18:45").expect("Valid time");
        assert_eq!(time_24h.hour24(), 18);
        assert_eq!(time_24h.minute(), 45);
    }

    #[test]
    fn test_weather_time_invalid() {
        assert!(WeatherTime::parse("invalid").is_err());
        assert!(WeatherTime::parse("25:00").is_err());
        assert!(WeatherTime::parse("12:60").is_err());
    }

    #[test]
    fn test_weather_condition_icon() {
        let condition = WeatherCondition::new("Clear".to_string());
        assert_eq!(condition.icon(), "☀️");
        assert_eq!(condition.to_string(), "Clear");

        let cloudy = WeatherCondition::new("Partly cloudy".to_string());
        assert_eq!(cloudy.icon(), "⛅");

        let rainy = WeatherCondition::new("Light rain".to_string());
        assert_eq!(rainy.icon(), "🌧️");
    }

    #[test]
    fn test_dew_point_calculation() {
        let temp = Temperature::new(20).unwrap();
        let humidity = Humidity::new(60.0).unwrap();

        let dew_point = humidity.dew_point(&temp);
        assert_eq!(dew_point.as_celsius(), 12);
    }

    #[test]
    fn test_day_length_calculation() {
        let sunrise = WeatherTime::parse("06:30 AM").unwrap();
        let sunset = WeatherTime::parse("06:30 PM").unwrap();
        let astronomy = Astronomy::new(sunrise, sunset);

        let day_length = astronomy.day_length();
        assert_eq!(day_length.hours(), 12);
        assert_eq!(day_length.minutes(), 0);
    }

    #[test]
    fn test_solar_noon_calculation() {
        let sunrise = WeatherTime::parse("06:30 AM").unwrap();
        let sunset = WeatherTime::parse("06:30 PM").unwrap();
        let astronomy = Astronomy::new(sunrise, sunset);

        let solar_noon = astronomy.solar_noon().unwrap();
        assert_eq!(solar_noon.hour24(), 12);
        assert_eq!(solar_noon.minute(), 30);
    }

    #[test]
    fn test_astronomy_creation_and_calculations() {
        let sunrise = WeatherTime::parse("06:00 AM").unwrap();
        let sunset = WeatherTime::parse("07:00 PM").unwrap();
        let astronomy = Astronomy::new(sunrise, sunset);

        // Test sunrise and sunset access
        assert_eq!(astronomy.sunrise().hour24(), 6);
        assert_eq!(astronomy.sunset().hour24(), 19);

        // Test that day length calculation works correctly
        assert_eq!(astronomy.day_length().hours(), 13);
        assert_eq!(astronomy.day_length().minutes(), 0);

        // Test solar noon calculation
        let solar_noon = astronomy.solar_noon().unwrap();
        assert_eq!(solar_noon.hour24(), 12);
        assert_eq!(solar_noon.minute(), 30);
    }

    #[test]
    fn test_last_updated_from_epoch() {
        let last_updated = LastUpdated::from_epoch(1673620200).expect("Valid timestamp");
        assert_eq!(last_updated.format_display(), "2023-01-13 14:30Z");
        assert_eq!(last_updated.to_string(), "2023-01-13 14:30Z");
    }

    #[test]
    fn test_last_updated_from_api_format() {
        let last_updated = LastUpdated::from_api_format("2023-01-13 14:30").expect("Valid format");
        assert_eq!(last_updated.format_display(), "2023-01-13 14:30Z");
    }

    #[test]
    fn test_last_updated_invalid_epoch() {
        // Test with truly invalid epoch timestamp (i64::MAX causes overflow in chrono)
        assert!(LastUpdated::from_epoch(i64::MAX).is_err());
    }

    #[test]
    fn test_last_updated_invalid_api_format() {
        // Test with various invalid formats
        assert!(LastUpdated::from_api_format("invalid").is_err());
        assert!(LastUpdated::from_api_format("2023-13-01 14:30").is_err()); // Invalid month
        assert!(LastUpdated::from_api_format("2023-01-32 14:30").is_err()); // Invalid day
        assert!(LastUpdated::from_api_format("2023-01-13 25:30").is_err()); // Invalid hour
        assert!(LastUpdated::from_api_format("2023-01-13 14:60").is_err()); // Invalid minute
    }

    #[test]
    fn test_last_updated_consistency() {
        // Test that epoch and API format produce same result for same timestamp
        let epoch_version = LastUpdated::from_epoch(1673620200).expect("Valid timestamp");
        let api_version = LastUpdated::from_api_format("2023-01-13 14:30").expect("Valid format");

        assert_eq!(epoch_version.format_display(), api_version.format_display());
        assert_eq!(epoch_version.to_string(), api_version.to_string());
    }

    #[test]
    fn test_wind_direction_creation_and_validation() {
        // Test valid compass directions
        let north = WindDirection::from_compass("N").expect("Valid direction");
        assert_eq!(north.to_string(), "N");

        let east = WindDirection::from_compass("E").expect("Valid direction");
        assert_eq!(east.to_string(), "E");

        let south = WindDirection::from_compass("S").expect("Valid direction");
        assert_eq!(south.to_string(), "S");

        let west = WindDirection::from_compass("W").expect("Valid direction");
        assert_eq!(west.to_string(), "W");

        let nnw = WindDirection::from_compass("NNW").expect("Valid direction");
        assert_eq!(nnw.to_string(), "NNW");
    }

    #[test]
    fn test_wind_direction_validation() {
        // Should reject invalid compass strings
        assert!(WindDirection::from_compass("INVALID").is_err());
        assert!(WindDirection::from_compass("X").is_err());
        assert!(WindDirection::from_compass("NORTH").is_err());

        // Should accept all valid compass directions
        assert!(WindDirection::from_compass("N").is_ok());
        assert!(WindDirection::from_compass("S").is_ok());
        assert!(WindDirection::from_compass("NNW").is_ok());
    }

    #[test]
    fn test_wind_direction_all_compass_points() {
        // Test all 16 compass points
        let directions = [
            "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
            "NW", "NNW",
        ];

        for direction in directions {
            let wind_dir = WindDirection::from_compass(direction).unwrap();
            assert_eq!(wind_dir.to_string(), direction);
        }
    }

    #[test]
    fn test_wind_direction_from_compass() {
        // Test creating from compass strings
        let north = WindDirection::from_compass("N").expect("Valid compass");
        assert_eq!(north.to_string(), "N");

        let northeast = WindDirection::from_compass("NE").expect("Valid compass");
        assert_eq!(northeast.to_string(), "NE");

        let south = WindDirection::from_compass("S").expect("Valid compass");
        assert_eq!(south.to_string(), "S");

        // Test case insensitive
        let west = WindDirection::from_compass("w").expect("Valid compass");
        assert_eq!(west.to_string(), "W");
    }

    #[test]
    fn test_wind_direction_from_compass_invalid() {
        // Should reject invalid compass strings
        assert!(WindDirection::from_compass("INVALID").is_err());
        assert!(WindDirection::from_compass("").is_err());
        assert!(WindDirection::from_compass("NORTH").is_err());
        assert!(WindDirection::from_compass("X").is_err());
    }

    #[test]
    fn test_wind_direction_edge_cases() {
        // Test edge cases for compass validation
        assert!(WindDirection::from_compass("").is_err()); // Empty string
        assert!(WindDirection::from_compass("  ").is_err()); // Whitespace only
        assert!(WindDirection::from_compass("North").is_err()); // Full word instead of abbreviation
    }

    #[test]
    fn test_zero_cost_abstractions() {
        // Verify that our phantom type approach has zero runtime cost
        let temp = Temperature::new(20).unwrap();
        assert_eq!(std::mem::size_of_val(&temp), std::mem::size_of::<i32>());

        let humidity = Humidity::new(60.0).unwrap();
        assert_eq!(std::mem::size_of_val(&humidity), std::mem::size_of::<f32>());

        let pressure = Pressure::new(1013).unwrap();
        assert_eq!(std::mem::size_of_val(&pressure), std::mem::size_of::<u32>());

        let wind_direction = WindDirection::from_compass("S").unwrap();
        assert_eq!(
            std::mem::size_of_val(&wind_direction),
            std::mem::size_of::<String>()
        );
    }

    // === Wind Speed Color Categorization Tests ===

    #[test]
    fn test_wind_speed_calm_category() {
        // Test calm winds (0-19 km/h) - white color
        let calm_zero = WindSpeed::new(0).unwrap();
        assert_eq!(calm_zero.category(), WindSpeedCategory::Calm);
        assert_eq!(calm_zero.color(), "#FFFFFF");

        let calm_mid = WindSpeed::new(10).unwrap();
        assert_eq!(calm_mid.category(), WindSpeedCategory::Calm);
        assert_eq!(calm_mid.color(), "#FFFFFF");

        let calm_max = WindSpeed::new(19).unwrap();
        assert_eq!(calm_max.category(), WindSpeedCategory::Calm);
        assert_eq!(calm_max.color(), "#FFFFFF");
    }

    #[test]
    fn test_wind_speed_moderate_breezes_category() {
        // Test moderate breezes (20-50 km/h) - green color
        let moderate_min = WindSpeed::new(20).unwrap();
        assert_eq!(moderate_min.category(), WindSpeedCategory::ModerateBreezes);
        assert_eq!(moderate_min.color(), "#00AA00");

        let moderate_mid = WindSpeed::new(35).unwrap();
        assert_eq!(moderate_mid.category(), WindSpeedCategory::ModerateBreezes);
        assert_eq!(moderate_mid.color(), "#00AA00");

        let moderate_max = WindSpeed::new(50).unwrap();
        assert_eq!(moderate_max.category(), WindSpeedCategory::ModerateBreezes);
        assert_eq!(moderate_max.color(), "#00AA00");
    }

    #[test]
    fn test_wind_speed_gales_category() {
        // Test gales (51-88 km/h) - orange color
        let gale_min = WindSpeed::new(51).unwrap();
        assert_eq!(gale_min.category(), WindSpeedCategory::Gales);
        assert_eq!(gale_min.color(), "#FFA500");

        let gale_mid = WindSpeed::new(70).unwrap();
        assert_eq!(gale_mid.category(), WindSpeedCategory::Gales);
        assert_eq!(gale_mid.color(), "#FFA500");

        let gale_max = WindSpeed::new(88).unwrap();
        assert_eq!(gale_max.category(), WindSpeedCategory::Gales);
        assert_eq!(gale_max.color(), "#FFA500");
    }

    #[test]
    fn test_wind_speed_storms_category() {
        // Test storms (89-117 km/h) - red color
        let storm_min = WindSpeed::new(89).unwrap();
        assert_eq!(storm_min.category(), WindSpeedCategory::Storms);
        assert_eq!(storm_min.color(), "#FF0000");

        let storm_mid = WindSpeed::new(100).unwrap();
        assert_eq!(storm_mid.category(), WindSpeedCategory::Storms);
        assert_eq!(storm_mid.color(), "#FF0000");

        let storm_max = WindSpeed::new(117).unwrap();
        assert_eq!(storm_max.category(), WindSpeedCategory::Storms);
        assert_eq!(storm_max.color(), "#FF0000");
    }

    #[test]
    fn test_wind_speed_hurricane_category() {
        // Test hurricane (118+ km/h) - purple color
        let hurricane_min = WindSpeed::new(118).unwrap();
        assert_eq!(hurricane_min.category(), WindSpeedCategory::Hurricane);
        assert_eq!(hurricane_min.color(), "#9B30FF");

        let hurricane_mid = WindSpeed::new(150).unwrap();
        assert_eq!(hurricane_mid.category(), WindSpeedCategory::Hurricane);
        assert_eq!(hurricane_mid.color(), "#9B30FF");

        let hurricane_max = WindSpeed::new(200).unwrap();
        assert_eq!(hurricane_max.category(), WindSpeedCategory::Hurricane);
        assert_eq!(hurricane_max.color(), "#9B30FF");
    }

    #[test]
    fn test_wind_speed_category_boundaries() {
        // Test exact boundary conditions
        assert_eq!(WindSpeed::new(19).unwrap().category(), WindSpeedCategory::Calm);
        assert_eq!(WindSpeed::new(20).unwrap().category(), WindSpeedCategory::ModerateBreezes);
        assert_eq!(WindSpeed::new(50).unwrap().category(), WindSpeedCategory::ModerateBreezes);
        assert_eq!(WindSpeed::new(51).unwrap().category(), WindSpeedCategory::Gales);
        assert_eq!(WindSpeed::new(88).unwrap().category(), WindSpeedCategory::Gales);
        assert_eq!(WindSpeed::new(89).unwrap().category(), WindSpeedCategory::Storms);
        assert_eq!(WindSpeed::new(117).unwrap().category(), WindSpeedCategory::Storms);
        assert_eq!(WindSpeed::new(118).unwrap().category(), WindSpeedCategory::Hurricane);
    }

    #[test]
    fn test_wind_speed_format_colored() {
        // Test that colored formatting produces correct Pango markup (only numbers colored)
        let calm = WindSpeed::new(10).unwrap();
        assert_eq!(
            calm.format_colored(),
            "<span foreground=\"#FFFFFF\">10</span> km/h"
        );

        let moderate = WindSpeed::new(30).unwrap();
        assert_eq!(
            moderate.format_colored(),
            "<span foreground=\"#00AA00\">30</span> km/h"
        );

        let gale = WindSpeed::new(60).unwrap();
        assert_eq!(
            gale.format_colored(),
            "<span foreground=\"#FFA500\">60</span> km/h"
        );

        let storm = WindSpeed::new(100).unwrap();
        assert_eq!(
            storm.format_colored(),
            "<span foreground=\"#FF0000\">100</span> km/h"
        );

        let hurricane = WindSpeed::new(150).unwrap();
        assert_eq!(
            hurricane.format_colored(),
            "<span foreground=\"#9B30FF\">150</span> km/h"
        );
    }

    #[test]
    fn test_wind_speed_format_colored_with_gusts() {
        // Test that sustained wind and gusts are colored separately based on their own categories
        let calm_with_moderate_gusts = WindSpeed::with_gusts(15, Some(45)).unwrap();
        assert_eq!(calm_with_moderate_gusts.category(), WindSpeedCategory::Calm);
        // Sustained: 15 km/h = Calm (white), Gusts: 45 km/h = Moderate Breezes (green)
        assert_eq!(
            calm_with_moderate_gusts.format_colored(),
            "<span foreground=\"#FFFFFF\">15</span> km/h (Gusts: <span foreground=\"#00AA00\">45</span> km/h)"
        );

        let moderate_with_gale_gusts = WindSpeed::with_gusts(25, Some(60)).unwrap();
        assert_eq!(moderate_with_gale_gusts.category(), WindSpeedCategory::ModerateBreezes);
        // Sustained: 25 km/h = Moderate Breezes (green), Gusts: 60 km/h = Gales (orange)
        assert_eq!(
            moderate_with_gale_gusts.format_colored(),
            "<span foreground=\"#00AA00\">25</span> km/h (Gusts: <span foreground=\"#FFA500\">60</span> km/h)"
        );
    }
}
