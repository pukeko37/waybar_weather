//! Presentation formatting functions for domain types.
//!
//! These functions produce Pango markup and emoji — concerns that belong
//! in the display layer rather than the domain.

use crate::domain::{WeatherCondition, WindSpeed, WindSpeedCategory};

/// Format wind speed with Pango color markup for Waybar tooltip.
/// Only colors the numbers, not the units.
pub fn format_wind_colored(wind: &WindSpeed) -> String {
    let sustained_color = wind.color();
    let sustained_colored = format!(
        "<span foreground=\"{}\">{}</span>",
        sustained_color,
        wind.sustained_value()
    );

    match wind.gusts_value() {
        Some(gusts) => {
            let gust_category = match gusts {
                0..=19 => WindSpeedCategory::Calm,
                20..=50 => WindSpeedCategory::ModerateBreezes,
                51..=88 => WindSpeedCategory::Gales,
                89..=117 => WindSpeedCategory::Storms,
                118.. => WindSpeedCategory::Hurricane,
            };
            let gust_color = gust_category.color();
            let gust_colored = format!("<span foreground=\"{}\">{}</span>", gust_color, gusts);

            format!(
                "{} km/h (Gusts: {} km/h)",
                sustained_colored, gust_colored
            )
        }
        None => format!("{} km/h", sustained_colored),
    }
}

/// Format wind speed compactly for title bar display (e.g., "43 km/h").
/// Only shows sustained wind speed, colored by category.
pub fn format_wind_colored_compact(wind: &WindSpeed) -> String {
    let sustained_color = wind.color();
    format!(
        "<span foreground=\"{}\">{}</span> km/h",
        sustained_color,
        wind.sustained_value()
    )
}

/// Get appropriate weather icon for a condition.
pub fn condition_icon(condition: &WeatherCondition) -> &'static str {
    let condition_lower = condition.description().to_lowercase();
    match condition_lower.as_str() {
        c if c.contains("sunny") || c.contains("clear") => "☀️",
        c if c.contains("partly") || c.contains("partial") => "⛅",
        c if c.contains("cloudy") || c.contains("overcast") => "☁️",
        c if c.contains("rain") || c.contains("drizzle") => "🌧️",
        c if c.contains("storm") || c.contains("thunder") => "⛈️",
        c if c.contains("snow") || c.contains("blizzard") => "🌨️",
        c if c.contains("fog") || c.contains("mist") => "🌫️",
        c if c.contains("wind") => "💨",
        _ => "🌤️",
    }
}
