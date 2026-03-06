//! Presentation formatting functions for domain types.
//!
//! These functions produce Pango markup and emoji — concerns that belong
//! in the display layer rather than the domain.

use crate::domain::{WeatherCondition, WindSpeed, WindSpeedCategory};

/// Get the Pango color string for a wind speed category.
fn category_color(category: &WindSpeedCategory) -> &'static str {
    match category {
        WindSpeedCategory::Calm => "#FFFFFF",
        WindSpeedCategory::ModerateBreezes => "#00AA00",
        WindSpeedCategory::Gales => "#FFA500",
        WindSpeedCategory::Storms => "#FF0000",
        WindSpeedCategory::Hurricane => "#9B30FF",
    }
}

/// Format wind speed with Pango color markup for Waybar tooltip.
/// Only colors the numbers, not the units.
pub fn format_wind_colored(wind: &WindSpeed) -> String {
    let sustained_color = category_color(&wind.category());
    let sustained_colored = format!(
        "<span foreground=\"{}\">{}</span>",
        sustained_color,
        wind.sustained_value()
    );

    match (wind.gusts_value(), wind.gust_category()) {
        (Some(gusts), Some(gust_cat)) => {
            let gust_color = category_color(&gust_cat);
            let gust_colored = format!("<span foreground=\"{}\">{}</span>", gust_color, gusts);

            format!(
                "{} km/h (Gusts: {} km/h)",
                sustained_colored, gust_colored
            )
        }
        _ => format!("{} km/h", sustained_colored),
    }
}

/// Format wind speed compactly for title bar display (e.g., "43 km/h").
/// Only shows sustained wind speed, colored by category.
pub fn format_wind_colored_compact(wind: &WindSpeed) -> String {
    let sustained_color = category_color(&wind.category());
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
