//! Application layer: orchestrates domain and display without infrastructure details.

use crate::display::{WaybarFormatter, WaybarOutput};
use crate::domain::WeatherFetcher;

/// Fetch weather data and format it for Waybar output.
///
/// Generic over the fetcher implementation, enabling test doubles.
pub fn fetch_and_format<F: WeatherFetcher>(
    fetcher: &F,
    formatter: &WaybarFormatter,
    location: &str,
) -> Result<WaybarOutput, anyhow::Error> {
    let weather_data = fetcher.fetch_weather(location)?;
    let output = formatter.format(&weather_data)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        CurrentWeather, Humidity, LastUpdated, Location, Pressure, Temperature, WeatherCondition,
        WeatherData, WeatherDay, WindDirection, WindSpeed,
    };

    struct StubWeatherFetcher {
        data: Result<WeatherData, anyhow::Error>,
    }

    impl WeatherFetcher for StubWeatherFetcher {
        fn fetch_weather(&self, _location: &str) -> Result<WeatherData, anyhow::Error> {
            match &self.data {
                Ok(_) => {
                    // Rebuild since WeatherData is not Clone
                    Ok(create_stub_weather_data())
                }
                Err(e) => Err(anyhow::anyhow!("{}", e)),
            }
        }
    }

    fn create_stub_weather_data() -> WeatherData {
        WeatherData {
            current: CurrentWeather {
                last_updated: LastUpdated::from_epoch(1673620200).unwrap(),
                temperature: Temperature::new(18).unwrap(),
                feels_like: Temperature::new(16).unwrap(),
                condition: WeatherCondition::new("Partly cloudy".to_string()),
                humidity: Humidity::new(72.0).unwrap(),
                wind_speed: WindSpeed::new(25).unwrap(),
                wind_direction: WindDirection::from_compass("SW").unwrap(),
                pressure: Pressure::new(1010).unwrap(),
            },
            location: Location::new("Wellington".to_string()),
            weather_day: Some(WeatherDay {
                astronomy: None,
                hourly_weather: vec![],
            }),
        }
    }

    #[test]
    fn test_fetch_and_format_success() {
        let fetcher = StubWeatherFetcher {
            data: Ok(create_stub_weather_data()),
        };
        let formatter = WaybarFormatter::new();

        let output = fetch_and_format(&fetcher, &formatter, "Wellington").unwrap();

        assert!(output.text.contains("18°C"));
        assert!(output.text.contains("Wellington"));
        assert!(output.tooltip.contains("Partly cloudy"));
        assert!(output.tooltip.contains("Humidity: 72%"));
    }

    #[test]
    fn test_fetch_and_format_error() {
        let fetcher = StubWeatherFetcher {
            data: Err(anyhow::anyhow!("connection refused")),
        };
        let formatter = WaybarFormatter::new();

        let result = fetch_and_format(&fetcher, &formatter, "Wellington");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("connection refused"));
    }

    #[test]
    fn test_fetch_and_format_with_wind() {
        let fetcher = StubWeatherFetcher {
            data: Ok(create_stub_weather_data()),
        };
        let formatter = WaybarFormatter::new();

        let output = fetch_and_format(&fetcher, &formatter, "Wellington").unwrap();

        // Wind 25 km/h is ModerateBreezes (green)
        assert!(output.tooltip.contains("<span foreground=\"#00AA00\">25</span> km/h"));
        assert!(output.tooltip.contains("SW"));
    }
}
