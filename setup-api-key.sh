#!/usr/bin/env bash
set -euo pipefail

# WeatherAPI.com API Key Setup Script for NixOS
# This script helps configure your WeatherAPI.com API key in the NixOS configuration

echo "🌤️  Weather App - API Key Setup for NixOS"
echo "=========================================="
echo

# Check if API key is already set
if [[ -n "${WEATHER_API_KEY:-}" ]]; then
    echo "✅ WEATHER_API_KEY is already set in your environment"
    echo "   Current key: ${WEATHER_API_KEY:0:8}..."
    echo
    read -p "Do you want to update it? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Keeping existing API key. Exiting."
        exit 0
    fi
fi

echo "📋 Instructions:"
echo "1. Sign up for a free account at: https://www.weatherapi.com/signup.aspx"
echo "2. After signup, find your API key in the account dashboard"
echo "3. Copy your API key and paste it below"
echo

read -p "🔑 Enter your WeatherAPI.com API key: " -r API_KEY

if [[ -z "$API_KEY" ]]; then
    echo "❌ No API key provided. Exiting."
    exit 1
fi

# Add API key to NixOS configuration
NIX_FILE="$(dirname "$0")/../users/andrew.nix"

if [[ ! -f "$NIX_FILE" ]]; then
    echo "❌ Could not find NixOS configuration file: $NIX_FILE"
    exit 1
fi

echo
echo "📝 Adding API key to NixOS configuration..."

# Create backup
cp "$NIX_FILE" "$NIX_FILE.backup.$(date +%Y%m%d_%H%M%S)"

# Replace the commented line or existing API key with the new one
if grep -q "WEATHER_API_KEY" "$NIX_FILE"; then
    # Replace existing line
    sed -i "s|WEATHER_API_KEY = \".*\";|WEATHER_API_KEY = \"$API_KEY\";|g" "$NIX_FILE"
else
    # Add new line after MOZ_ENABLE_WAYLAND
    sed -i "/MOZ_ENABLE_WAYLAND = \"1\";/a\\      WEATHER_API_KEY = \"$API_KEY\";" "$NIX_FILE"
fi

echo "✅ API key added to NixOS configuration: $NIX_FILE"
echo

echo "🧪 Testing API key..."

# Test the API key
if command -v curl >/dev/null 2>&1; then
    TEST_URL="http://api.weatherapi.com/v1/current.json?key=$API_KEY&q=Wellington&aqi=no"
    if curl -s --max-time 10 "$TEST_URL" | grep -q '"name"'; then
        echo "✅ API key is working correctly!"
    else
        echo "⚠️  API key test failed. Please check:"
        echo "   - Your API key is correct"
        echo "   - You have internet connectivity"
        echo "   - WeatherAPI.com service is available"
    fi
else
    echo "ℹ️  curl not found - skipping API key test"
fi

echo
echo "📦 Next steps:"
echo "1. Rebuild your NixOS configuration:"
echo "   sudo nixos-rebuild switch"
echo
echo "2. After rebuilding, restart your desktop session or waybar:"
echo "   systemctl --user restart waybar"
echo

echo "🎉 Setup complete!"
echo
echo "The weather app will now work with Waybar. Your weather widget should show:"
echo "- Current temperature and conditions with emoji"
echo "- Detailed tooltip with humidity, wind, pressure, and more"
echo "- Hourly forecast for today"
echo "- Sunrise, sunset, and day length information"
echo
echo "If you encounter issues, check the logs:"
echo "   journalctl --user -u waybar -f"
