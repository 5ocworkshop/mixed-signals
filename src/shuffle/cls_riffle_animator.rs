// <FILE>src/shuffle/cls_riffle_animator.rs</FILE> - <DESC>Stepped riffle shuffle animator for frame-by-frame animation</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Implementing shuffle algorithms phases 2-4</WCTX>
// <CLOG>Initial implementation of stepped riffle animator</CLOG>

use crate::rng::Rng;
use std::collections::VecDeque;

/// Animation state for riffle shuffle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiffleState {
    /// About to cut the deck
    Cutting,
    /// Dropping cards from left and right halves
    Dropping,
    /// Animation complete
    Complete,
}

/// Stepped riffle shuffle animator for frame-by-frame visualization.
///
/// Unlike [`riffle_shuffle`](super::riffle_shuffle) which performs the shuffle
/// immediately, this struct allows stepping through the animation one card at
/// a time for visual effects.
///
/// # Usage Pattern
///
/// ```rust
/// use mixed_signals::rng::Rng;
/// use mixed_signals::shuffle::RiffleAnimator;
///
/// let mut rng = Rng::with_seed(42);
/// let deck: Vec<i32> = (1..=10).collect();
/// let mut animator = RiffleAnimator::new(deck, 1, &mut rng);
///
/// while animator.step() {
///     // Render current state: animator.items()
///     // animator.state() tells you what phase we're in
/// }
///
/// let shuffled = animator.into_items();
/// ```
///
/// # Animation Phases
///
/// 1. **Cutting**: Deck is split into two halves
/// 2. **Dropping**: Cards drop one at a time from alternating halves
/// 3. **Complete**: All cards dropped, pass complete (may restart for more passes)
#[derive(Debug, Clone)]
pub struct RiffleAnimator<T> {
    /// Current visible state
    items: Vec<T>,
    /// Left half (being depleted during drop)
    left: VecDeque<T>,
    /// Right half (being depleted during drop)
    right: VecDeque<T>,
    /// Result being built
    result: Vec<T>,
    /// Pre-computed drop decisions (true = drop from left)
    drop_from_left: Vec<bool>,
    /// Index into drop decisions
    drop_index: usize,
    /// Remaining passes
    passes_remaining: usize,
    /// Current animation state
    state: RiffleState,
    /// Stored RNG seed for deterministic replay
    seed: u64,
}

impl<T: Clone> RiffleAnimator<T> {
    /// Create a new riffle animator.
    ///
    /// Random decisions are pre-computed for deterministic replay.
    pub fn new(items: Vec<T>, passes: usize, rng: &mut Rng) -> Self {
        let len = items.len();
        let seed = rng.uniform(0.0, u32::MAX as f32) as u64;

        let mut animator = Self {
            items,
            left: VecDeque::new(),
            right: VecDeque::new(),
            result: Vec::new(),
            drop_from_left: Vec::new(),
            drop_index: 0,
            passes_remaining: passes,
            state: if passes == 0 || len <= 1 {
                RiffleState::Complete
            } else {
                RiffleState::Cutting
            },
            seed,
        };

        if animator.state != RiffleState::Complete {
            animator.prepare_pass(rng);
        }

        animator
    }

    /// Prepare decisions for a single pass.
    fn prepare_pass(&mut self, rng: &mut Rng) {
        let len = self.items.len();
        if len <= 1 {
            self.state = RiffleState::Complete;
            return;
        }

        // Cut point
        let cut_min = (len / 4).max(1);
        let cut_max = (3 * len / 4).min(len - 1);
        let cut = (rng.uniform(cut_min as f32, cut_max as f32).floor() as usize).clamp(1, len - 1);

        self.left = self.items[..cut].iter().cloned().collect();
        self.right = self.items[cut..].iter().cloned().collect();
        self.result.clear();
        self.drop_from_left.clear();
        self.drop_index = 0;

        // Pre-compute all drop decisions
        let mut l_remaining = self.left.len();
        let mut r_remaining = self.right.len();

        while l_remaining > 0 || r_remaining > 0 {
            let total = l_remaining + r_remaining;
            let p_left = l_remaining as f32 / total as f32;
            let from_left = rng.uniform(0.0, 1.0) < p_left && l_remaining > 0;

            if from_left {
                l_remaining -= 1;
            } else if r_remaining > 0 {
                r_remaining -= 1;
            } else {
                l_remaining -= 1;
            }
            self.drop_from_left
                .push(from_left || r_remaining == 0 && l_remaining > 0);
        }

        self.state = RiffleState::Dropping;
    }

    /// Advance animation by one step.
    ///
    /// Returns `true` if animation is still in progress, `false` when complete.
    pub fn step(&mut self) -> bool {
        match self.state {
            RiffleState::Complete => false,

            RiffleState::Cutting => {
                // Cutting happens in prepare_pass, just transition
                self.state = RiffleState::Dropping;
                true
            }

            RiffleState::Dropping => {
                if self.drop_index >= self.drop_from_left.len() {
                    // Pass complete
                    self.items = std::mem::take(&mut self.result);
                    self.passes_remaining -= 1;

                    if self.passes_remaining == 0 {
                        self.state = RiffleState::Complete;
                        return false;
                    }

                    // Prepare next pass (need fresh RNG state)
                    let mut rng =
                        Rng::with_seed(self.seed.wrapping_add(self.passes_remaining as u64));
                    self.prepare_pass(&mut rng);
                    return true;
                }

                // Drop one card
                let from_left = self.drop_from_left[self.drop_index];
                if from_left {
                    if let Some(value) = self.left.pop_front() {
                        self.result.push(value);
                    } else if let Some(value) = self.right.pop_front() {
                        self.result.push(value);
                    }
                } else if let Some(value) = self.right.pop_front() {
                    self.result.push(value);
                } else if let Some(value) = self.left.pop_front() {
                    self.result.push(value);
                }

                self.drop_index += 1;
                true
            }
        }
    }

    /// Get current visible state of items.
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Get current animation state.
    pub fn state(&self) -> RiffleState {
        self.state
    }

    /// Check if animation is complete.
    pub fn is_complete(&self) -> bool {
        self.state == RiffleState::Complete
    }

    /// Consume animator and return final shuffled items.
    pub fn into_items(self) -> Vec<T> {
        self.items
    }
}

// <FILE>src/shuffle/cls_riffle_animator.rs</FILE> - <DESC>Stepped riffle shuffle animator for frame-by-frame animation</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
