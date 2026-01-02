// <FILE>mixed-signals/src/traits/mod.rs</FILE> - <DESC>Signal traits module</DESC>
// <VERS>VERSION: 2.2.0</VERS>
// <WCTX>Bipolar core refactor - normalization layer</WCTX>
// <CLOG>Added NormalizedFrom export for explicit range normalization</CLOG>

mod ext_signal;
mod fnc_signal;
mod signal;

pub use ext_signal::{Map, NormalizedFrom, SignalExt};
pub use fnc_signal::{Fn1, Fn2};
pub use signal::SignalTime;
pub use signal::{Phase, Signal, SignalContext, SignalRange};

// <FILE>mixed-signals/src/traits/mod.rs</FILE> - <DESC>Signal traits module</DESC>
// <VERS>END OF VERSION: 2.2.0</VERS>
