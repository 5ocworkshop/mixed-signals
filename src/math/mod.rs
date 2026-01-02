// <FILE>mixed-signals/src/math/mod.rs</FILE> - <DESC>Mathematical utilities module</DESC>
// <VERS>VERSION: 1.3.0</VERS>
// <WCTX>Deduplication refactor</WCTX>
// <CLOG>Added harmonic_sin_cos and harmonic_phase for physics trig stability</CLOG>

pub mod fnc_cpu_features;
pub mod fnc_cubic_bezier;
mod fnc_derive_seed;
pub mod fnc_fast_random;
pub mod fnc_fast_random_batch;
mod fnc_harmonic;
pub mod fnc_quadratic_bezier;
mod fnc_sanitize;

pub use fnc_cpu_features::{detect_cpu_features, has_avx2, has_fma, CpuFeatures};
pub use fnc_cubic_bezier::{bezier_x, bezier_x_derivative, bezier_y, solve_bezier};
pub(crate) use fnc_derive_seed::derive_seed;
pub use fnc_fast_random::fast_random;
pub use fnc_fast_random_batch::fast_random_batch;
pub use fnc_harmonic::{harmonic_phase, harmonic_sin_cos};
pub use fnc_quadratic_bezier::quadratic_bezier;
pub(crate) use fnc_sanitize::{finite_or, finite_or_clamp, finite_or_f64, finite_or_min};

// <FILE>mixed-signals/src/math/mod.rs</FILE> - <DESC>Mathematical utilities module</DESC>
// <VERS>END OF VERSION: 1.3.0</VERS>
