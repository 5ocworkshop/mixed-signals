// <FILE>mixed-signals/src/noise/mod.rs</FILE> - <DESC>Noise generators module</DESC>
// <VERS>VERSION: 1.0.0 - 2025-12-22</VERS>
// <WCTX>Signal generator implementation</WCTX>
// <CLOG>Initial implementation</CLOG>

//! Noise generators for continuous stochastic signals.
//!
//! Invalid inputs (NaN/Inf) are sanitized to defaults at sample time to keep
//! outputs finite. For valid finite inputs, behavior is unchanged.

mod cls_perlin;
mod cls_white_noise;

pub use cls_perlin::PerlinNoise;
pub use cls_white_noise::WhiteNoise;

// <FILE>mixed-signals/src/noise/mod.rs</FILE> - <DESC>Noise generators module</DESC>
// <VERS>END OF VERSION: 1.0.0 - 2025-12-22</VERS>
