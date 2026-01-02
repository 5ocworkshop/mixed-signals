// <FILE>src/visualization/mod.rs</FILE> - <DESC>Signal visualization module</DESC>
// <VERS>VERSION: 2.0.1</VERS>
// <WCTX>mixed-signals extraction: bgraph integration</WCTX>
// <CLOG>Fixed SignalView wrapper to work with bgraph OFPF refactor</CLOG>

//! Signal visualization using an internal SignalView widget.

mod cls_signal_view;

pub use cls_signal_view::{ColorGradient, RenderMode, SignalView};

// <FILE>src/visualization/mod.rs</FILE> - <DESC>Signal visualization module</DESC>
// <VERS>END OF VERSION: 2.0.1</VERS>
