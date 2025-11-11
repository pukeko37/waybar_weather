# waybar_weather - Waybar Weather Widget

A Rust-based weather data fetcher for Waybar that retrieves weather information from WeatherAPI.com and outputs Waybar-compatible JSON format.

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

## Installation

### For Nix Users

This project provides a Nix flake for reproducible builds and easy integration with NixOS.

#### Quick Start with Nix

```bash
# Run directly from GitHub
nix run github:pukeko37/waybar_weather -- "Wellington"

# Build locally
nix build

# The binary will be available at ./result/bin/waybar_weather
./result/bin/waybar_weather "Auckland"
```

#### Add to NixOS Configuration

Add this flake as an input in your NixOS configuration:

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    waybar-weather.url = "github:pukeko37/waybar_weather";
  };

  outputs = { self, nixpkgs, waybar-weather, ... }: {
    nixosConfigurations.yourhost = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        {
          environment.systemPackages = [
            waybar-weather.packages.x86_64-linux.default
          ];
        }
      ];
    };
  };
}
```

#### Use in Home Manager

```nix
# home.nix
{ inputs, pkgs, ... }: {
  home.packages = [
    inputs.waybar-weather.packages.${pkgs.system}.default
  ];

  # Set API key
  home.sessionVariables = {
    WEATHER_API_KEY = "your_api_key_here";  # Or use a secret management solution
  };
}
```

#### Development Shell

Enter a development environment with all required tools:

```bash
nix develop

# Now you have cargo, rust-analyzer, and other tools available
cargo build
cargo test
```

### Building with Cargo

```bash
cargo build --release
```

The binary will be available at `target/release/waybar_weather`.

## Usage

First, set your API key as an environment variable:

```bash
export WEATHER_API_KEY="your_api_key_here"
```

Then run the program:

```bash
# Default location (Wellington)
./target/release/waybar_weather

# Custom location
./target/release/waybar_weather "Auckland"
./target/release/waybar_weather "London"
./target/release/waybar_weather "New York"
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

- `ureq` - Synchronous HTTP client with JSON support
- `serde` and `serde_json` - JSON serialization/deserialization
- `time` - Date and time handling
- `anyhow` - Error handling with context
- `urlencoding` - URL encoding for location names

## Waybar Configuration

To use waybar_weather in your Waybar setup:

1. Get a free API key from WeatherAPI.com
2. Build the release binary (or install via Nix)
3. Set the `WEATHER_API_KEY` environment variable in your shell profile
4. Configure your Waybar to use the binary

Example Waybar config:
```json
{
    "custom/weather": {
        "format": "{}",
        "exec": "/path/to/waybar_weather/target/release/waybar_weather",
        "interval": 1800,
        "return-type": "json"
    }
}
```

Or for Nix users with the package installed:
```json
{
    "custom/weather": {
        "format": "{}",
        "exec": "waybar_weather Wellington",
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

## Performance

- Binary size: ~2.3MB (Cargo release build), ~2.6MB (Nix build with optimizations)
- No caching mechanism (fetches fresh data each time)
- 10-second timeout for API requests
- Synchronous HTTP client for simplicity and smaller binary size
- Minimal memory usage and fast execution
- Type-safe domain modeling with zero-cost abstractions