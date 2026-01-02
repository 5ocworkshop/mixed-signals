// <FILE>mixed-signals/src/processing/mod.rs</FILE> - <DESC>Signal processing operators module</DESC>
// <VERS>VERSION: 1.4.0</VERS>
// <WCTX>Bipolar core refactor - normalization layer</WCTX>
// <CLOG>Added Normalized wrapper for converting any signal to 0..1 range</CLOG>

mod cls_abs;
mod cls_biquad;
mod cls_clamp;
mod cls_clipper;
mod cls_invert;
mod cls_lowpass;
mod cls_normalized;
mod cls_quantize;
mod cls_remap;
mod cls_svf;
mod fnc_bipolar_helpers;

pub use cls_abs::Abs;
pub use cls_biquad::{Biquad, BiquadMode};
pub use cls_clamp::Clamp;
pub use cls_clipper::{ClipMode, Clipper};
pub use cls_invert::Invert;
pub use cls_lowpass::LowPass;
pub use cls_normalized::Normalized;
pub use cls_quantize::Quantize;
pub use cls_remap::Remap;
pub use cls_svf::{Svf, SvfFixed, SvfMode};
pub use fnc_bipolar_helpers::{bipolar_to_unipolar, remap_range, unipolar_to_bipolar};

// <FILE>mixed-signals/src/processing/mod.rs</FILE> - <DESC>Signal processing operators module</DESC>
// <VERS>END OF VERSION: 1.4.0</VERS>
