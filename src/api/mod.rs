//! API module for external weather service integration with type-safe parsing.

pub mod client;
pub mod models;

pub use client::*;

#[cfg(test)]
mod tests {
    use crate::api::models::*;

    #[test]
    fn test_weather_response_parsing() {
        let json_data = r#"
        {
            "location": {
                "name": "Wellington",
                "localtime": "2023-01-13 14:30"
            },
            "current": {
                "last_updated_epoch": 1673620200,
                "last_updated": "2023-01-13 14:30",
                "temp_c": 20.0,
                "temp_f": 68.0,
                "is_day": 1,
                "condition": {
                    "text": "Clear",
                    "icon": "//cdn.weatherapi.com/weather/64x64/day/113.png",
                    "code": 1000
                },
                "wind_mph": 9.4,
                "wind_kph": 15.1,
                "wind_degree": 315,
                "wind_dir": "NW",
                "pressure_mb": 1013.0,
                "pressure_in": 29.91,
                "precip_mm": 0.0,
                "precip_in": 0.0,
                "humidity": 60,
                "cloud": 0,
                "feelslike_c": 22.0,
                "feelslike_f": 71.6,
                "vis_km": 10.0,
                "vis_miles": 6.0,
                "uv": 6.0,
                "gust_mph": 18.8,
                "gust_kph": 30.2
            },
            "forecast": {
                "forecastday": [
                    {
                        "astro": {
                            "sunrise": "06:30 AM",
                            "sunset": "06:30 PM"
                        },
                        "hour": [
                            {
                                "time": "2023-01-13 12:00",
                                "temp_c": 22.0,
                                "condition": {
                                    "text": "Sunny"
                                },
                                "wind_kph": 10.0,
                                "wind_dir": "N",
                                "gust_kph": 18.0
                            }
                        ]
                    }
                ]
            }
        }
        "#;

        let response: WeatherApiResponse = serde_json::from_str(json_data).expect("Valid JSON");
        let weather_data: WeatherData = response.try_into().expect("Valid domain conversion");

        assert_eq!(weather_data.current.temperature.as_celsius(), 20);
        assert_eq!(weather_data.current.humidity.as_int(), 60);
        assert_eq!(weather_data.location.to_string(), "Wellington");
    }

    #[test]
    fn test_current_weather_parsing() {
        let current_json = r#"
        {
            "last_updated_epoch": 1673620200,
            "last_updated": "2023-01-13 14:30",
            "temp_c": 18.5,
            "condition": {
                "text": "Partly cloudy"
            },
            "wind_kph": 12.0,
            "wind_dir": "SW",
            "pressure_mb": 1010.0,
            "humidity": 70,
            "feelslike_c": 18.0,
            "gust_kph": 24.0
        }
        "#;

        let current: CurrentApi = serde_json::from_str(current_json).expect("Valid JSON");
        let domain_current: CurrentWeather = current.try_into().expect("Valid domain conversion");

        assert_eq!(domain_current.temperature.as_celsius(), 19); // Rounded from 18.5
        assert_eq!(domain_current.humidity.as_int(), 70);
        assert_eq!(domain_current.condition.to_string(), "Partly cloudy");
        assert_eq!(domain_current.wind_direction.to_string(), "SW");
    }

    #[test]
    fn test_hourly_weather_parsing() {
        let hourly_json = r#"
        {
            "time": "2023-01-13 18:00",
            "temp_c": 18.0,
            "condition": {
                "text": "Partly cloudy"
            },
            "wind_kph": 12.0,
            "wind_dir": "SW",
            "gust_kph": 20.0
        }
        "#;

        let hourly: HourApi = serde_json::from_str(hourly_json).expect("Valid JSON");
        let domain_hourly: HourlyWeather = hourly.try_into().expect("Valid domain conversion");

        assert_eq!(domain_hourly.time.hour24(), 18);
        assert_eq!(domain_hourly.temperature.as_celsius(), 18);
        assert_eq!(domain_hourly.condition.to_string(), "Partly cloudy");
        assert_eq!(
            domain_hourly.wind_speed.to_string(),
            "12 km/h (Gusts: 20 km/h)"
        );
    }

    #[test]
    fn test_astronomy_parsing() {
        let astro_json = r#"
        {
            "sunrise": "06:15 AM",
            "sunset": "08:45 PM"
        }
        "#;

        let astro: AstroApi = serde_json::from_str(astro_json).expect("Valid JSON");
        let domain_astro: crate::domain::Astronomy = astro.try_into().expect("Valid domain conversion");

        assert_eq!(domain_astro.sunrise().hour24(), 6);
        assert_eq!(domain_astro.sunrise().minute(), 15);
        assert_eq!(domain_astro.sunset().hour24(), 20);
        assert_eq!(domain_astro.sunset().minute(), 45);
    }

    #[test]
    fn test_invalid_temperature_handling() {
        let invalid_temp_json = r#"
        {
            "last_updated_epoch": 1673620200,
            "last_updated": "2023-01-13 14:30",
            "temp_c": 999.0,
            "condition": {
                "text": "Clear"
            },
            "wind_kph": 15.1,
            "wind_dir": "NW",
            "pressure_mb": 1013.0,
            "humidity": 60,
            "feelslike_c": 999.0,
            "gust_kph": 30.2
        }
        "#;

        let current: CurrentApi = serde_json::from_str(invalid_temp_json).expect("Valid JSON");
        let result: Result<CurrentWeather, _> = current.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_time_parsing_variations() {
        let time_variations = vec![
            ("2023-01-13 06:00", 6),
            ("2023-01-13 12:00", 12),
            ("2023-01-13 18:00", 18),
            ("2023-01-13 00:30", 0),
        ];

        for (time_str, expected_hour) in time_variations {
            let hourly = HourApi {
                time: time_str.to_string(),
                temp_c: 20.0,
                condition: ConditionApi {
                    text: "Clear".to_string(),
                },
                wind_kph: 10.0,
                wind_dir: "N".to_string(),
                gust_kph: 19.0,
            };

            let domain_hourly: HourlyWeather = hourly.try_into().expect("Valid conversion");
            assert_eq!(domain_hourly.time.hour24(), expected_hour);
        }
    }

    #[test]
    fn test_wind_speed_with_gusts() {
        let hourly_with_gusts = HourApi {
            time: "2023-01-13 15:00".to_string(),
            temp_c: 20.0,
            condition: ConditionApi {
                text: "Windy".to_string(),
            },
            wind_kph: 30.0,
            wind_dir: "W".to_string(),
            gust_kph: 50.0,
        };

        let domain_hourly: HourlyWeather = hourly_with_gusts.try_into().expect("Valid conversion");
        assert_eq!(
            domain_hourly.wind_speed.to_string(),
            "30 km/h (Gusts: 50 km/h)"
        );
    }
}
