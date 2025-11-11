# Weather - Waybar Weather Widget

A Rust implementation of a weather data fetcher that replaces the original `weather.sh` script for Waybar. This executable fetches weather data from WeatherAPI.com and outputs Waybar-compatible JSON format.

## Features

- Fetches weather data from WeatherAPI.com API
- Outputs Waybar-compatible JSON format with text and tooltip
- Supports custom location (defaults to Wellington, NZ)
- Comprehensive weather information including:
  - Current temperature and conditions
  - Humidity and dew point calculation
  - Wind speed, direction, and gusts
  - Atmospheric pressure
  - Sunrise/sunset times
  - Solar zenith calculation
  - Day length calculation
  - Hourly forecast for today
- Weather condition emoji mapping
- Robust error handling with informative messages
- No caching (fetches fresh data on each call)

## Prerequisites

You need a free API key from WeatherAPI.com:

1. Visit [https://www.weatherapi.com/](https://www.weatherapi.com/)
2. Sign up for a free account
3. Get your API key from the dashboard
4. Set the `WEATHER_API_KEY` environment variable

## Building

```bash
cargo build --release
```

The binary will be available at `target/release/weather`.

## Usage

First, set your API key as an environment variable:

```bash
export WEATHER_API_KEY="your_api_key_here"
```

Then run the program:

```bash
# Default location (Wellington)
./target/release/weather

# Custom location
./target/release/weather "Auckland"
./target/release/weather "London"
./target/release/weather "New York"
```

## Environment Variables

- `WEATHER_API_KEY` (Required) - Your WeatherAPI.com API key

## Output Format

The program outputs JSON in the format expected by Waybar:

```json
{
  "text": "☀️ 20°C Wellington",
  "tooltip": "📍 Location: Wellington\n🌡️ Temperature: 20°C\n..."
}
```

### Text Format
- Weather emoji based on conditions
- Temperature in Celsius
- Location name

### Tooltip Information
- Location
- Current temperature and "feels like" temperature
- Weather condition description
- Humidity percentage and calculated dew point
- Wind speed, direction, and gusts (when available)
- Atmospheric pressure
- Sunrise and sunset times
- Solar zenith time (solar noon)
- Day length
- Hourly forecast for the current day
- Last updated timestamp

## Weather Icon Mapping

The program maps weather conditions to appropriate emoji:

- ☀️ Sunny/Clear conditions
- ⛅ Partly cloudy/Partial conditions  
- ☁️ Cloudy/Overcast conditions
- 🌧️ Rain/Drizzle
- ⛈️ Storms/Thunder
- 🌨️ Snow/Blizzard
- 🌫️ Fog/Mist
- 💨 Windy conditions
- 🌤️ Default/Unknown conditions

## Error Handling

If the weather data cannot be fetched, the program outputs an error message in Waybar format:

```json
{
  "text": "🌤️ -- Weather unavailable",
  "tooltip": "Unable to fetch weather data for [Location]\n\nError: [error details]\nService: WeatherAPI.com\n\nLast attempt: [timestamp]"
}
```

Common errors:
- Missing `WEATHER_API_KEY` environment variable
- Invalid API key
- Network connectivity issues
- Invalid location

## Dependencies

- `reqwest` - HTTP client with rustls-tls for secure connections
- `serde` and `serde_json` - JSON serialization/deserialization
- `tokio` - Async runtime
- `chrono` - Date and time handling
- `anyhow` - Error handling
- `urlencoding` - URL encoding for location names

## Replacing the Shell Script

To replace the existing `weather.sh` in your Waybar configuration:

1. Get a free API key from WeatherAPI.com
2. Build the release binary
3. Copy or symlink the binary to your waybar-resources directory
4. Set the `WEATHER_API_KEY` environment variable in your shell profile
5. Update your Waybar configuration to use the new binary instead of `weather.sh`

Example Waybar config:
```json
{
    "custom/weather": {
        "format": "{}",
        "exec": "/path/to/weather/target/release/weather",
        "interval": 1800,
        "return-type": "json"
    }
}
```

For systemd users, you may want to set the environment variable in your systemd user environment:
```bash
systemctl --user set-environment WEATHER_API_KEY="your_api_key_here"
```

## Testing

Run the test suite with:

```bash
cargo test
```

For integration tests that require network access, set your API key:
```bash
WEATHER_API_KEY="your_api_key_here" cargo test
```

Tests include:
- Weather icon mapping
- Dew point calculation
- Time parsing and calculations  
- Day length and zenith calculations
- API response parsing
- Mock data processing
- Integration test (requires internet connection and API key)

## API Limits

WeatherAPI.com free tier provides:
- 1,000,000 calls per month
- Current weather data
- 3-day forecast
- Historical weather (last 7 days)
- Astronomy data

This should be more than sufficient for personal use with Waybar updates every 30 minutes.

## Migration from wttr.in

This version has been migrated from wttr.in to WeatherAPI.com for better reliability and more comprehensive data. The main changes:

- **Requires API key**: You need to sign up for a free account
- **Better reliability**: Commercial API with better uptime
- **More data fields**: Additional weather information available
- **Same output format**: Drop-in replacement for existing Waybar configs

## Performance

- Binary size: ~5.6MB (release build)
- No caching mechanism (fetches fresh data each time)
- 10-second timeout for API requests
- Uses rustls instead of system OpenSSL for better portability
- Minimal memory usage and fast execution