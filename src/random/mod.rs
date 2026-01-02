// <FILE>mixed-signals/src/random/mod.rs</FILE> - <DESC>Random signal generators module</DESC>
// <VERS>VERSION: 1.3.0</VERS>
// <WCTX>Audio synthesis primitives</WCTX>
// <CLOG>Added ImpulseNoise and StudentTNoise generators</CLOG>

//! Random signal generators for stochastic and noise-based effects.
//!
//! This module provides deterministic random value generators using various
//! distributions and approaches. All signals are stateless and reproducible
//! given the same seed.
//!
//! ## Standard vs Fast variants
//!
//! Standard types (`SeededRandom`, `PinkNoise`, etc.) use ChaCha8Rng for
//! cryptographic-quality randomness. Fast variants (`FastSeededRandom`,
//! `FastPinkNoise`, etc.) use SplitMix64 hashing for ~25x better performance.
//!
//! Use Fast variants for animation/visualization where speed matters more than
//! cryptographic quality. Both are deterministic and reproducible.
//!
//! Invalid inputs (NaN/Inf) are sanitized to defaults at sample time to keep
//! outputs finite. For valid finite inputs, behavior is unchanged.

mod cls_correlated_noise;
mod cls_gaussian_noise;
mod cls_impulse_noise;
mod cls_per_character_noise;
mod cls_pink_noise;
mod cls_poisson_noise;
mod cls_seeded_random;
mod cls_spatial_noise;
mod cls_student_t_noise;
mod fnc_hash_to_index;

// Fast variants using hash-based RNG
mod cls_fast_correlated_noise;
mod cls_fast_pink_noise;
mod cls_fast_seeded_random;

pub use cls_correlated_noise::CorrelatedNoise;
pub use cls_gaussian_noise::GaussianNoise;
pub use cls_impulse_noise::ImpulseNoise;
pub use cls_per_character_noise::PerCharacterNoise;
pub use cls_pink_noise::PinkNoise;
pub use cls_poisson_noise::PoissonNoise;
pub use cls_seeded_random::SeededRandom;
pub use cls_spatial_noise::SpatialNoise;
pub use cls_student_t_noise::StudentTNoise;
pub use fnc_hash_to_index::hash_to_index;

// Fast variants
pub use cls_fast_correlated_noise::FastCorrelatedNoise;
pub use cls_fast_pink_noise::FastPinkNoise;
pub use cls_fast_seeded_random::FastSeededRandom;

// <FILE>mixed-signals/src/random/mod.rs</FILE> - <DESC>Random signal generators module</DESC>
// <VERS>END OF VERSION: 1.3.0</VERS>
