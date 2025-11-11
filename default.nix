{ lib, rustPlatform }:

rustPlatform.buildRustPackage {
  pname = "waybar_weather";
  version = "0.1.0";

  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };

  meta = with lib; {
    description = "Waybar weather widget using WeatherAPI.com";
    longDescription = ''
      A Rust-based weather data fetcher for Waybar that retrieves weather
      information from WeatherAPI.com and outputs Waybar-compatible JSON format.
      Provides comprehensive weather data including temperature, humidity, wind,
      pressure, sunrise/sunset times, and hourly forecasts with type-safe domain
      modeling and zero-cost abstractions.
    '';
    homepage = "https://github.com/pukeko37/waybar_weather";
    license = licenses.mit;
    maintainers = [ ];
    platforms = platforms.linux;
  };
}
