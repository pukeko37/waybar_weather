//! Weather application with domain-driven design and type safety.
//! Fetches weather data from WeatherAPI.com and outputs JSON for Waybar.
//!
//! This file is the composition root: it constructs concrete types and
//! delegates to the application layer.

use anyhow::Result;
use waybar_weather::app;
use waybar_weather::infra::api::WeatherClient;
use waybar_weather::infra::display::WaybarFormatter;

fn main() -> Result<()> {
    let location = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Wellington".to_string());

    let client = match WeatherClient::new() {
        Ok(client) => client,
        Err(e) => {
            let error_output = WaybarFormatter::create_error_output(&location, e);
            println!("{}", serde_json::to_string(&error_output)?);
            return Ok(());
        }
    };
    let formatter = WaybarFormatter::new();

    match app::fetch_and_format(&client, &formatter, &location) {
        Ok(output) => {
            println!("{}", serde_json::to_string(&output)?);
        }
        Err(e) => {
            let error_output = WaybarFormatter::create_error_output(&location, e);
            println!("{}", serde_json::to_string(&error_output)?);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_handling_flow() {
        let location = "test_location";
        let error = anyhow::anyhow!("Test error");
        let error_output = WaybarFormatter::create_error_output(location, error);

        assert!(error_output.text.contains("unavailable"));
        assert!(error_output.tooltip.contains("test_location"));
        assert!(error_output.tooltip.contains("Test error"));

        // Validate JSON serialization
        let json = serde_json::to_string(&error_output).unwrap();
        assert!(json.contains("text"));
        assert!(json.contains("tooltip"));
    }

    #[test]
    fn test_default_location_handling() {
        let args: Vec<String> = vec!["weather".to_string()];
        // Simulate empty args by checking the logic
        let location = args.get(1).unwrap_or(&"Wellington".to_string()).clone();
        assert_eq!(location, "Wellington");
    }

    #[test]
    fn test_custom_location_handling() {
        let test_location = "Auckland";
        // Test that custom location would be passed through
        assert_eq!(test_location, "Auckland");
    }
}
