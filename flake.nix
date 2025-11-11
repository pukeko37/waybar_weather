{
  description = "Weather data fetcher for Waybar";

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
          pname = "weather";
          version = "0.1.0";

          src = ./.;

          cargoLock = { lockFile = ./Cargo.lock; };

          meta = with pkgs.lib; {
            description =
              "Weather data fetcher for Waybar that replaces weather.sh";
            longDescription = ''
              A Rust implementation of a weather data fetcher that replaces the original
              weather.sh script for Waybar. This executable fetches weather data from
              wttr.in and outputs Waybar-compatible JSON format with comprehensive
              weather information including temperature, humidity, wind, pressure,
              sunrise/sunset times, and hourly forecasts.
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
            echo "Weather app development environment"
            echo "Rust toolchain: ${rustToolchain}"
            echo "Run 'cargo build' to build the project"
            echo "Run 'cargo run -- Wellington' to test with Wellington weather"
          '';
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/weather";
        };
      });
}
