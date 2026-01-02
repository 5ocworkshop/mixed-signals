// <FILE>src/shuffle/cls_overhand_animator.rs</FILE> - <DESC>Stepped overhand shuffle animator for frame-by-frame animation</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Implementing shuffle algorithms phases 2-4</WCTX>
// <CLOG>Initial implementation of stepped overhand animator</CLOG>

use crate::rng::Rng;
use std::collections::VecDeque;

/// Animation state for overhand shuffle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverhandState {
    /// Taking a chunk from the back
    TakingChunk,
    /// Placing chunk on top
    PlacingChunk,
    /// Animation complete
    Complete,
}

/// Stepped overhand shuffle animator for frame-by-frame visualization.
///
/// Unlike [`overhand_shuffle`](super::overhand_shuffle) which performs the shuffle
/// immediately, this struct allows stepping through the animation one chunk at
/// a time for visual effects.
///
/// # Usage Pattern
///
/// ```rust
/// use mixed_signals::rng::Rng;
/// use mixed_signals::shuffle::OverhandAnimator;
///
/// let mut rng = Rng::with_seed(42);
/// let deck: Vec<i32> = (1..=10).collect();
/// let mut animator = OverhandAnimator::new(deck, 1, &mut rng);
///
/// while animator.step() {
///     // Render current state: animator.items()
///     // animator.current_chunk() shows the chunk being moved
/// }
///
/// let shuffled = animator.into_items();
/// ```
///
/// # Animation Phases
///
/// 1. **TakingChunk**: A chunk is being cut from the back
/// 2. **PlacingChunk**: The chunk is being placed on top
/// 3. **Complete**: All chunks placed, pass complete
#[derive(Debug, Clone)]
pub struct OverhandAnimator<T> {
    /// Current visible state
    items: Vec<T>,
    /// Cards remaining in hand (back of deck)
    hand: Vec<T>,
    /// Result pile (front of deck)
    result: VecDeque<T>,
    /// Current chunk being transferred
    current_chunk: Vec<T>,
    /// Pre-computed chunk sizes for this pass
    chunk_sizes: Vec<usize>,
    /// Index into chunk_sizes
    chunk_index: usize,
    /// Remaining passes
    passes_remaining: usize,
    /// Current animation state
    state: OverhandState,
    /// Stored RNG seed for deterministic replay
    seed: u64,
}

impl<T: Clone> OverhandAnimator<T> {
    /// Create a new overhand animator.
    ///
    /// Random decisions are pre-computed for deterministic replay.
    pub fn new(items: Vec<T>, passes: usize, rng: &mut Rng) -> Self {
        let len = items.len();
        let seed = rng.uniform(0.0, u32::MAX as f32) as u64;

        let mut animator = Self {
            items,
            hand: Vec::new(),
            result: VecDeque::new(),
            current_chunk: Vec::new(),
            chunk_sizes: Vec::new(),
            chunk_index: 0,
            passes_remaining: passes,
            state: if passes == 0 || len <= 1 {
                OverhandState::Complete
            } else {
                OverhandState::TakingChunk
            },
            seed,
        };

        if animator.state != OverhandState::Complete {
            animator.prepare_pass(rng);
        }

        animator
    }

    /// Prepare decisions for a single pass.
    fn prepare_pass(&mut self, rng: &mut Rng) {
        let len = self.items.len();
        if len <= 1 {
            self.state = OverhandState::Complete;
            return;
        }

        self.hand = self.items.clone();
        self.result.clear();
        self.chunk_sizes.clear();
        self.chunk_index = 0;

        // Pre-compute chunk sizes
        let mut remaining = len;
        while remaining > 0 {
            let avg_chunk = (remaining / 5).max(1);
            let u = rng.uniform(0.01, 1.0);
            let raw_size = (-u.ln() * avg_chunk as f32).ceil() as usize;
            let chunk_size = raw_size.clamp(1, remaining);
            self.chunk_sizes.push(chunk_size);
            remaining -= chunk_size;
        }

        self.state = OverhandState::TakingChunk;
    }

    /// Advance animation by one step.
    ///
    /// Returns `true` if animation is still in progress, `false` when complete.
    pub fn step(&mut self) -> bool {
        match self.state {
            OverhandState::Complete => false,

            OverhandState::TakingChunk => {
                if self.chunk_index >= self.chunk_sizes.len() {
                    // Pass complete
                    self.items = self.result.drain(..).collect();
                    self.passes_remaining -= 1;

                    if self.passes_remaining == 0 {
                        self.state = OverhandState::Complete;
                        return false;
                    }

                    // Prepare next pass
                    let mut rng =
                        Rng::with_seed(self.seed.wrapping_add(self.passes_remaining as u64));
                    self.prepare_pass(&mut rng);
                    return true;
                }

                // Take chunk from back of hand
                let chunk_size = self.chunk_sizes[self.chunk_index];
                let chunk_start = self.hand.len().saturating_sub(chunk_size);
                self.current_chunk = self.hand.drain(chunk_start..).collect();

                self.state = OverhandState::PlacingChunk;
                true
            }

            OverhandState::PlacingChunk => {
                // Place current chunk on top of result without extra clones
                while let Some(value) = self.current_chunk.pop() {
                    self.result.push_front(value);
                }
                self.chunk_index += 1;

                self.state = OverhandState::TakingChunk;
                true
            }
        }
    }

    /// Get current visible state of items.
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Get current chunk being transferred (may be empty).
    pub fn current_chunk(&self) -> &[T] {
        &self.current_chunk
    }

    /// Get current animation state.
    pub fn state(&self) -> OverhandState {
        self.state
    }

    /// Check if animation is complete.
    pub fn is_complete(&self) -> bool {
        self.state == OverhandState::Complete
    }

    /// Consume animator and return final shuffled items.
    pub fn into_items(self) -> Vec<T> {
        self.items
    }
}

// <FILE>src/shuffle/cls_overhand_animator.rs</FILE> - <DESC>Stepped overhand shuffle animator for frame-by-frame animation</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
