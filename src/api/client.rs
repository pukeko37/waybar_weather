//! HTTP client for fetching weather data from WeatherAPI.com API.

use crate::api::models::{WeatherApiResponse, WeatherData};

use anyhow::{Context, Result};
use std::time::Duration;

/// Weather API client for WeatherAPI.com service
pub struct WeatherClient {
    agent: ureq::Agent,
    base_url: String,
    api_key: String,
}

impl WeatherClient {
    /// Create a new weather client with API key from environment
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("WEATHER_API_KEY")
            .context("WEATHER_API_KEY environment variable not set. Get your free API key from https://www.weatherapi.com/")?;

        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(10))
            .build();

        Ok(Self {
            agent,
            base_url: "http://api.weatherapi.com/v1".to_string(),
            api_key,
        })
    }

    /// Create a new weather client with explicit API key (for testing)
    #[cfg(test)]
    pub fn with_api_key(api_key: String) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(10))
            .build();

        Self {
            agent,
            base_url: "http://api.weatherapi.com/v1".to_string(),
            api_key,
        }
    }

    /// Fetch weather data for a location
    pub fn fetch_weather(&self, location: &str) -> Result<WeatherData> {
        // Use forecast endpoint with days=1 to get current weather + today's astronomy/hourly data
        let url = format!(
            "{}/forecast.json?key={}&q={}&days=1&aqi=no&alerts=no",
            self.base_url,
            self.api_key,
            self.format_location(location)
        );

        let response = self
            .agent
            .get(&url)
            .call()
            .with_context(|| format!("Failed to send request to: {}", url))?;

        if response.status() != 200 {
            let status = response.status();
            let error_text = response.into_string().unwrap_or_default();
            anyhow::bail!(
                "Weather API returned error status {}: {}. Response: {}",
                status,
                url,
                error_text
            );
        }

        let api_response: WeatherApiResponse = response
            .into_json()
            .context("Failed to parse JSON response from weather API")?;

        api_response
            .try_into()
            .context("Failed to convert API response to domain model")
    }

    /// Format location for URL (encode spaces and special characters)
    fn format_location(&self, location: &str) -> String {
        urlencoding::encode(location.trim()).to_string()
    }
}

impl std::fmt::Debug for WeatherClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeatherClient")
            .field("base_url", &self.base_url)
            .field("api_key", &"[REDACTED]")
            .finish()
    }
}

impl Default for WeatherClient {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback for tests or when API key is not available
            let agent = ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(10))
                .build();

            Self {
                agent,
                base_url: "http://api.weatherapi.com/v1".to_string(),
                api_key: "test_key".to_string(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_location() {
        let client = WeatherClient::with_api_key("test_key".to_string());

        assert_eq!(client.format_location("Wellington"), "Wellington");
        assert_eq!(client.format_location("New York"), "New%20York");
        assert_eq!(client.format_location(" London "), "London");
        assert_eq!(client.format_location("São Paulo"), "S%C3%A3o%20Paulo");
    }

    #[test]
    fn test_client_creation_with_api_key() {
        let client = WeatherClient::with_api_key("test_api_key".to_string());
        assert_eq!(client.base_url, "http://api.weatherapi.com/v1");
        assert_eq!(client.api_key, "test_api_key");
    }

    #[test]
    fn test_client_creation_requires_api_key() {
        // Remove any existing API key
        std::env::remove_var("WEATHER_API_KEY");

        let result = WeatherClient::new();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("WEATHER_API_KEY environment variable not set"));
    }

    #[test]
    fn test_client_creation_with_env_var() {
        // Store original value if it exists
        let original_key = std::env::var("WEATHER_API_KEY").ok();

        std::env::set_var("WEATHER_API_KEY", "env_test_key");

        let client = WeatherClient::new().unwrap();
        assert_eq!(client.api_key, "env_test_key");

        // Restore original value or remove
        match original_key {
            Some(key) => std::env::set_var("WEATHER_API_KEY", key),
            None => std::env::remove_var("WEATHER_API_KEY"),
        }
    }

    #[test]
    fn test_fetch_weather_integration() {
        // Integration test - only runs when API key is available and not in CI
        if std::env::var("CI").is_ok() {
            return;
        }

        // Try to get API key from environment
        if let Ok(api_key) = std::env::var("WEATHER_API_KEY") {
            let client = WeatherClient::with_api_key(api_key);

            match client.fetch_weather("Wellington") {
                Ok(weather_data) => {
                    // Basic validation that we got weather data
                    assert!(!weather_data.location.to_string().is_empty());
                    // Temperature should be reasonable for Earth
                    assert!(weather_data.current.temperature.as_celsius() >= -40);
                    assert!(weather_data.current.temperature.as_celsius() <= 55);
                }
                Err(e) => {
                    // Log error but don't fail test in case of network issues
                    eprintln!("Integration test warning (network issues expected): {}", e);
                }
            }
        } else {
            eprintln!("Skipping integration test - no WEATHER_API_KEY environment variable");
        }
    }

    #[test]
    fn test_fetch_weather_invalid_api_key() {
        // Skip in CI environments
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
}
