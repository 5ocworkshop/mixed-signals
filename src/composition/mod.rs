// <FILE>mixed-signals/src/composition/mod.rs</FILE> - <DESC>Signal composition operators module</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Deduplication refactor</WCTX>
// <CLOG>Removed redundant Sum/Scale (use Add/Multiply instead)</CLOG>

mod cls_add;
mod cls_frequency_mod;
mod cls_mix;
mod cls_multiply;
mod cls_vca_centered;

pub use cls_add::Add;
pub use cls_frequency_mod::FrequencyMod;
pub use cls_mix::Mix;
pub use cls_multiply::Multiply;
pub use cls_vca_centered::VcaCentered;

/// Type alias for backward compatibility - use `Add` instead.
#[deprecated(since = "2.0.0", note = "Use Add instead - Sum was identical")]
pub type Sum<A, B> = Add<A, B>;

/// Type alias for backward compatibility - use `Multiply` instead.
#[deprecated(since = "2.0.0", note = "Use Multiply instead - Scale was identical")]
pub type Scale<A, B> = Multiply<A, B>;

// <FILE>mixed-signals/src/composition/mod.rs</FILE> - <DESC>Signal composition operators module</DESC>
// <VERS>END OF VERSION: 2.0.0</VERS>
