{
  description = "Waybar weather widget - fetches data from WeatherAPI.com";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default;
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "waybar_weather";
          version = "0.1.0";

          src = ./.;

          cargoLock = { lockFile = ./Cargo.lock; };

          meta = with pkgs.lib; {
            description = "Waybar weather widget using WeatherAPI.com";
            longDescription = ''
              A Rust-based weather data fetcher for Waybar that retrieves weather
              information from WeatherAPI.com and outputs Waybar-compatible JSON format.
              Provides comprehensive weather data including temperature, humidity, wind,
              pressure, sunrise/sunset times, and hourly forecasts with type-safe domain
              modeling and zero-cost abstractions.
            '';
            license = licenses.mit;
            maintainers = [ ];
            platforms = platforms.linux;
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.rust-analyzer
          ];

          shellHook = ''
            echo "waybar_weather development environment"
            echo "Rust toolchain: ${rustToolchain}"
            echo "Run 'cargo build' to build the project"
            echo "Run 'cargo run -- Wellington' to fetch Wellington weather"
            echo "Set WEATHER_API_KEY environment variable before running"
          '';
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/waybar_weather";
        };
      });
}
