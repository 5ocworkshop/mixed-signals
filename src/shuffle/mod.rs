// <FILE>src/shuffle/mod.rs</FILE> - <DESC>Shuffle algorithm module orchestrator</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Implementing shuffle algorithms phases 2-4</WCTX>
// <CLOG>Added Phase 2-4 algorithms: weighted, constrained, interleave, riffle, overhand, reservoir, smooth, and animators</CLOG>

//! Shuffle algorithms for collections.
//!
//! This module provides various shuffle algorithms optimized for different use cases
//! in UI work, gaming, and simulations. All algorithms are deterministic when used
//! with a seeded [`Rng`](crate::rng::Rng).
//!
//! # Algorithms
//!
//! | Algorithm | Time | Space | Use Case |
//! |-----------|------|-------|----------|
//! | [`fisher_yates`] | O(n) | O(1) | General-purpose, unbiased shuffle |
//! | [`partial_shuffle`] | O(k) | O(1) | "Draw k cards" without full shuffle |
//! | [`shuffle_copy`] | O(n) | O(n) | Non-mutating shuffle |
//! | [`sattolo`] | O(n) | O(1) | Cyclic permutation (no fixed points) |
//! | [`weighted_shuffle`] | O(n log n) | O(n) | Priority-biased ordering |
//! | [`constrained_shuffle`] | O(n²) | O(n) | Variety enforcement (max consecutive) |
//! | [`interleave`] | O(n) | O(n) | Deterministic Faro shuffle |
//! | [`riffle_shuffle`] | O(n) | O(n) | Realistic card riffle |
//! | [`overhand_shuffle`] | O(n) | O(n) | Casual card shuffle |
//! | [`reservoir_shuffle`] | O(n) | O(n) | Streaming/iterator input |
//! | [`smooth_shuffle`] | O(n²) | O(n) | Minimize transition jarring |
//!
//! # Animation Structs
//!
//! For frame-by-frame shuffle visualization:
//! - [`RiffleAnimator`] - Stepped riffle shuffle
//! - [`OverhandAnimator`] - Stepped overhand shuffle
//!
//! # Example
//!
//! ```rust
//! use mixed_signals::rng::Rng;
//! use mixed_signals::shuffle::{fisher_yates, sattolo, partial_shuffle};
//!
//! let mut rng = Rng::with_seed(42);
//! let mut deck = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
//!
//! // Standard shuffle
//! fisher_yates(&mut deck, &mut rng);
//!
//! // Draw 3 cards (only shuffles first 3 positions)
//! let mut deck2 = vec![1, 2, 3, 4, 5];
//! partial_shuffle(&mut deck2, 3, &mut rng);
//! let hand: Vec<_> = deck2[..3].to_vec();
//!
//! // Secret Santa: no one gets themselves
//! let mut assignments = vec!["Alice", "Bob", "Carol", "Dave"];
//! sattolo(&mut assignments, &mut rng);
//! ```

// Phase 1: Core
mod fnc_fisher_yates;
mod fnc_partial_shuffle;
mod fnc_sattolo;
mod fnc_shuffle_copy;

// Phase 2: Gaming
mod fnc_constrained_shuffle;
mod fnc_weighted_shuffle;

// Phase 3: Animation
mod cls_overhand_animator;
mod cls_riffle_animator;
mod fnc_interleave;
mod fnc_overhand_shuffle;
mod fnc_riffle_shuffle;

// Phase 4: Advanced
mod fnc_reservoir_shuffle;
mod fnc_smooth_shuffle;

// Phase 1 exports
pub use fnc_fisher_yates::fisher_yates;
pub use fnc_partial_shuffle::partial_shuffle;
pub use fnc_sattolo::sattolo;
pub use fnc_shuffle_copy::shuffle_copy;

// Phase 2 exports
pub use fnc_constrained_shuffle::constrained_shuffle;
pub use fnc_weighted_shuffle::weighted_shuffle;

// Phase 3 exports
pub use cls_overhand_animator::{OverhandAnimator, OverhandState};
pub use cls_riffle_animator::{RiffleAnimator, RiffleState};
pub use fnc_interleave::interleave;
pub use fnc_overhand_shuffle::overhand_shuffle;
pub use fnc_riffle_shuffle::riffle_shuffle;

// Phase 4 exports
pub use fnc_reservoir_shuffle::reservoir_shuffle;
pub use fnc_smooth_shuffle::smooth_shuffle;

// <FILE>src/shuffle/mod.rs</FILE> - <DESC>Shuffle algorithm module orchestrator</DESC>
// <VERS>END OF VERSION: 2.0.0</VERS>
