{ lib, rustPlatform }:

rustPlatform.buildRustPackage {
  pname = "waybar_weather";
  version = "0.1.0";

  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };

  meta = with lib; {
    description = "Weather data fetcher for Waybar that replaces weather.sh";
    longDescription = ''
      A Rust implementation of a weather data fetcher that replaces the original
      weather.sh script for Waybar. This executable fetches weather data from
      wttr.in and outputs Waybar-compatible JSON format with comprehensive
      weather information including temperature, humidity, wind, pressure,
      sunrise/sunset times, and hourly forecasts.
    '';
    homepage = "https://github.com/yourusername/nixos-config";
    license = licenses.mit;
    maintainers = [ ];
    platforms = platforms.linux;
  };
}
