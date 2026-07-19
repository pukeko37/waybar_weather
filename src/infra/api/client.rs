//! HTTP client for fetching weather data from WeatherAPI.com API.

use crate::app::WeatherFetcher;
use crate::infra::api::models::WeatherApiResponse;
use crate::domain::models::WeatherData;

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

    /// Create a new weather client with an explicit API key, bypassing the
    /// `WEATHER_API_KEY` environment read. Public so integration tests in
    /// `tests/` (which link against this crate as an ordinary library, not
    /// under `cfg(test)`) can construct a client without touching the
    /// process environment.
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

impl WeatherFetcher for WeatherClient {
    fn fetch_weather(&self, location: &str) -> Result<WeatherData> {
        self.fetch_weather(location)
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

}
