// <FILE>mixed-signals/src/envelopes/mod.rs</FILE> - <DESC>Envelope generators module</DESC>
// <VERS>VERSION: 1.0.0 - 2025-12-22</VERS>
// <WCTX>Signal generator implementation</WCTX>
// <CLOG>Initial implementation</CLOG>

//! Envelope generators for time-shaped control signals.
//!
//! Invalid inputs (NaN/Inf) are sanitized to defaults at sample time to keep
//! outputs finite. For valid finite inputs, behavior is unchanged.

mod cls_adsr;
mod cls_impact;
mod cls_linear;

pub use cls_adsr::Adsr;
pub use cls_impact::Impact;
pub use cls_linear::LinearEnvelope;

// <FILE>mixed-signals/src/envelopes/mod.rs</FILE> - <DESC>Envelope generators module</DESC>
// <VERS>END OF VERSION: 1.0.0 - 2025-12-22</VERS>
