// <FILE>mixed-signals/src/generators/mod.rs</FILE> - <DESC>Oscillator generators module</DESC>
// <VERS>VERSION: 1.3.0</VERS>
// <WCTX>Audio synthesis primitives - arbitrary curve support</WCTX>
// <CLOG>Added Keyframes for piecewise linear interpolation between keyframe points</CLOG>

//! Oscillator and utility signal generators.
//!
//! Invalid inputs (NaN/Inf) are sanitized to defaults at sample time to keep
//! outputs finite. For valid finite inputs, behavior is unchanged.

mod cls_constant;
mod cls_keyframes;
mod cls_phase_accumulator;
mod cls_phase_sine;
mod cls_pulse;
mod cls_ramp;
mod cls_sawtooth;
mod cls_sine;
mod cls_square;
mod cls_step;
mod cls_triangle;

pub use cls_constant::Constant;
pub use cls_keyframes::{Keyframe, Keyframes};
pub use cls_phase_accumulator::PhaseAccumulator;
pub use cls_phase_sine::PhaseSine;
pub use cls_pulse::Pulse;
pub use cls_ramp::Ramp;
pub use cls_sawtooth::Sawtooth;
pub use cls_sine::Sine;
pub use cls_square::Square;
pub use cls_step::Step;
pub use cls_triangle::Triangle;

// <FILE>mixed-signals/src/generators/mod.rs</FILE> - <DESC>Oscillator generators module</DESC>
// <VERS>END OF VERSION: 1.3.0</VERS>
