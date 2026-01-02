// <FILE>src/lib.rs</FILE> - <DESC>Signal generator library for animations, audio, games, and simulations</DESC>
// <VERS>VERSION: 1.7.0</VERS>
// <WCTX>Pre-launch fixes</WCTX>
// <CLOG>Updated docs for bipolar core architecture</CLOG>

//! # mixed-signals
//!
//! Primitives for signals, waveforms, noise, easing, RNG, and shuffling.
//!
//! ## Core Philosophy: Bipolar Core, Explicit Normalization
//!
//! **All oscillators and noise generators output bipolar `[-1, 1]` by default.**
//! This matches audio/synthesis conventions and enables mathematically correct composition.
//!
//! For TUI animation (opacity, brightness, progress bars), call `.normalized()` to get `[0, 1]`:
//!
//! ```rust
//! use mixed_signals::prelude::*;
//!
//! // Core signals are bipolar [-1, 1]
//! let sine = Sine::default();
//! assert_eq!(sine.sample(0.25), 1.0);  // Peak at +1
//! assert_eq!(sine.sample(0.75), -1.0); // Trough at -1
//!
//! // For TUI work, normalize to [0, 1]
//! let opacity = sine.normalized();
//! assert_eq!(opacity.sample(0.0), 0.5);  // Center maps to 0.5
//! assert_eq!(opacity.sample(0.25), 1.0); // Peak maps to 1.0
//! ```
//!
//! ## Why "mixed-signals"?
//!
//! This library mixes signal generators with utilities commonly needed alongside them:
//! shuffles, easing functions, physics solvers, and deterministic RNG. Built for terminal
//! animations, but the primitives have broad application.
//!
//! ## Signal Categories
//!
//! - **Generators**: Sine, Triangle, Square, Sawtooth, Pulse, Step, Ramp, Constant, Keyframes
//! - **Noise**: WhiteNoise, PerlinNoise, PinkNoise, CorrelatedNoise, SpatialNoise
//! - **Random**: GaussianNoise, PoissonNoise, PerCharacterNoise, ImpulseNoise, StudentTNoise
//! - **Envelopes**: ADSR, Linear, Impact
//! - **Physics**: DampedSpring, BouncingDrop, FrictionDecay, Pendulum, Orbit, Projectile, Attractor
//! - **Composition**: Add, Multiply, Mix, Scale, Sum, FrequencyMod
//! - **Processing**: Normalized, Abs, Invert, Clamp, Remap, Quantize
//! - **Shuffle**: fisher_yates, sattolo, weighted, constrained, riffle, overhand, and more
//!
//! ## Quick Start
//!
//! ```rust
//! use mixed_signals::prelude::*;
//!
//! // Oscillators output bipolar [-1, 1]
//! let sine = Sine::new(2.0, 1.0, 0.0, 0.0);
//! let bipolar_value = sine.sample(0.25);
//!
//! // For TUI: normalize to [0, 1]
//! let opacity = sine.normalized();
//! let tui_value = opacity.sample(0.25);
//!
//! // Compose signals
//! let noise = PerlinNoise::with_seed(42);
//! let organic = sine.mix(noise, 0.2).normalized();
//!
//! // Physics-based animation
//! let bounce = BouncingDrop::rubber_ball(0.0, 300.0, 500.0);
//! let y_position = bounce.sample(0.3);
//! ```
//!
//! ## RNG API
//!
//! Use [`rng::Rng`] for traditional random number generation:
//!
//! ```rust
//! use mixed_signals::rng::Rng;
//!
//! let mut rng = Rng::with_seed(42);
//! let dice = rng.uniform(1.0, 7.0).floor() as i32; // 1-6
//! let hit = rng.chance(0.7); // 70% probability
//! let color = rng.choose(&["red", "green", "blue"]);
//! ```
//!
//! ## Context-Aware Signals
//!
//! Signals can use runtime context for deterministic, reproducible behavior:
//!
//! ```rust
//! use mixed_signals::prelude::*;
//!
//! let noise = PerCharacterNoise::with_seed(99);
//! let ctx = SignalContext::new(100, 42).with_char_index(5);
//! // Same context always produces same value
//! let value = noise.sample_with_context(0.5, &ctx);
//! ```
pub mod composition;
pub mod core;
pub mod easing;
pub mod envelopes;
pub mod generators;
pub mod math;
pub mod noise;
pub mod physics;
pub mod processing;
pub mod random;
pub mod rng;
pub mod shuffle;
pub mod traits;
pub mod types;
#[cfg(feature = "visualization")]
pub mod visualization;
pub mod prelude {
    //! Convenient re-exports for common usage.
    pub use crate::composition::*;
    pub use crate::easing::{ease, EasingType};
    pub use crate::envelopes::*;
    pub use crate::generators::*;
    pub use crate::math::{
        bezier_x, bezier_x_derivative, bezier_y, quadratic_bezier, solve_bezier,
    };
    pub use crate::noise::*;
    pub use crate::physics::*;
    pub use crate::processing::*;
    pub use crate::random::*;
    pub use crate::rng::Rng;
    pub use crate::traits::{Phase, Signal, SignalContext, SignalExt, SignalRange, SignalTime};
    pub use crate::types::{SignalOrFloat, SignalSpec};
    #[cfg(feature = "visualization")]
    pub use crate::visualization::{RenderMode, SignalView};
}

// <FILE>src/lib.rs</FILE> - <DESC>Signal generator library for animations, audio, games, and simulations</DESC>
// <VERS>END OF VERSION: 1.7.0</VERS>
