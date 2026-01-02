// <FILE>mixed-signals/src/types/mod.rs</FILE> - <DESC>Types module</DESC>
// <VERS>VERSION: 1.1.0</VERS>
// <WCTX>Signal generator implementation + signal-driven parameters</WCTX>
// <CLOG>Added SignalOrFloat for flexible parameter specification</CLOG>

mod signal_or_float;
mod signal_spec;

pub use signal_or_float::SignalOrFloat;
pub use signal_spec::{SignalBuildError, SignalSpec};

// <FILE>mixed-signals/src/types/mod.rs</FILE> - <DESC>Types module</DESC>
// <VERS>END OF VERSION: 1.1.0</VERS>
