//! Library surface for `waybar_weather`: the domain/app/infra ring layering,
//! exposed so `tests/` can exercise the network-facing integration tests
//! against the real public API surface — the `waybar_weather` binary
//! (`main.rs`) is a thin composition-root wrapper over this.

pub mod app;
pub mod domain;
pub mod infra;
