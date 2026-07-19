//! Integration tests against the real WeatherAPI.com service.
//!
//! Network-dependent: skipped in CI, and skipped or best-effort when
//! `WEATHER_API_KEY` is absent from the environment, since these exercise
//! an external boundary rather than pure logic (see the `rust-style` skill's
//! test-driven-development section on why these live here, not in `src/`).

use waybar_weather::app;
use waybar_weather::infra::api::WeatherClient;
use waybar_weather::infra::display::WaybarFormatter;

#[test]
fn test_full_weather_flow() {
    if std::env::var("CI").is_ok() || std::env::var("WEATHER_API_KEY").is_err() {
        return;
    }

    let client = WeatherClient::new().expect("Failed to create client in test");
    let formatter = WaybarFormatter::new();

    match app::fetch_and_format(&client, &formatter, "Wellington") {
        Ok(output) => {
            assert!(!output.text.is_empty());
            assert!(!output.tooltip.is_empty());
            assert!(output.text.contains("°C"));

            // Validate JSON serialization
            let json = serde_json::to_string(&output).unwrap();
            assert!(json.contains("text"));
            assert!(json.contains("tooltip"));
        }
        Err(e) => {
            eprintln!("Integration test warning (network issues expected): {}", e);
        }
    }
}

#[test]
fn test_fetch_weather_integration() {
    if std::env::var("CI").is_ok() {
        return;
    }

    if let Ok(api_key) = std::env::var("WEATHER_API_KEY") {
        let client = WeatherClient::with_api_key(api_key);

        match client.fetch_weather("Wellington") {
            Ok(weather_data) => {
                assert!(!weather_data.location.to_string().is_empty());
                assert!(weather_data.current.temperature.as_celsius() >= -40);
                assert!(weather_data.current.temperature.as_celsius() <= 55);
            }
            Err(e) => {
                eprintln!("Integration test warning (network issues expected): {}", e);
            }
        }
    } else {
        eprintln!("Skipping integration test - no WEATHER_API_KEY environment variable");
    }
}

#[test]
fn test_fetch_weather_invalid_api_key() {
    if std::env::var("CI").is_ok() {
        return;
    }

    let client = WeatherClient::with_api_key("invalid_key".to_string());

    let result = client.fetch_weather("Wellington");
    assert!(result.is_err());

    let error_message = result.unwrap_err().to_string();
    // WeatherAPI returns 401 or 403 for invalid API key
    assert!(
        error_message.contains("401")
            || error_message.contains("403")
            || error_message.contains("invalid")
    );
}
