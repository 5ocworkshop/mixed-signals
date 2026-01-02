// <FILE>src/physics/mod.rs</FILE> - <DESC>Physics solvers module</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Physics module implementation</WCTX>
// <CLOG>Initial module hub with all physics components</CLOG>

//! Physics solvers for UI animations and simulations.
//!
//! This module provides deterministic physics solvers that implement the `Signal` trait,
//! enabling seamless composition with existing signal infrastructure.
//!
//! All solvers use analytical solutions (where possible) for framerate-independent behavior.
//!
//! # Components
//!
//! - [`DampedSpring`] - Harmonic motion with configurable damping
//! - [`BallisticTrajectory`] - Projectile motion under gravity
//! - [`FrictionDecay`] - Exponential velocity decay (scrolling, flinging)
//! - [`SimplePendulum`] - Pendulum oscillation
//! - [`CircularOrbit`] - Uniform circular motion
//! - [`PointAttractor`] - Force field toward a point
//! - [`BouncingDrop`] - Multi-bounce with energy loss
//!
//! # Example
//!
//! ```rust
//! use mixed_signals::physics::{DampedSpring, BouncingDrop};
//! use mixed_signals::traits::Signal;
//!
//! // Bouncy spring animation
//! let spring = DampedSpring::default();
//! let displacement = spring.sample(0.5);
//!
//! // Modal drop-in animation
//! let drop = BouncingDrop::rubber_ball(0.0, 300.0, 500.0);
//! let height = drop.sample(0.3);
//! ```

mod cls_attractor;
mod cls_bounce;
mod cls_decay;
mod cls_orbit;
mod cls_pendulum;
mod cls_projectile;
mod cls_spring;

pub use cls_attractor::PointAttractor;
pub use cls_bounce::BouncingDrop;
pub use cls_decay::FrictionDecay;
pub use cls_orbit::CircularOrbit;
pub use cls_pendulum::SimplePendulum;
pub use cls_projectile::BallisticTrajectory;
pub use cls_spring::DampedSpring;

// <FILE>src/physics/mod.rs</FILE> - <DESC>Physics solvers module</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
