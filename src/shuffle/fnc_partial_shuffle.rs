// <FILE>src/shuffle/fnc_partial_shuffle.rs</FILE> - <DESC>Partial shuffle for top-k selection</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Implementing shuffle algorithms for UI and gaming</WCTX>
// <CLOG>Initial implementation</CLOG>

use crate::rng::Rng;

/// Partial shuffle - only randomize the first k elements.
///
/// Performs a partial Fisher-Yates shuffle, randomizing only the first `k`
/// positions. This is more efficient than a full shuffle when you only need
/// a random subset (e.g., drawing cards from a deck).
///
/// After calling, `slice[0..k]` contains `k` uniformly random elements from
/// the original slice. Elements at positions `k..` may have been swapped but
/// are not guaranteed to be shuffled.
///
/// # Algorithm
///
/// - Time complexity: O(k)
/// - Space complexity: O(1) - in-place
/// - First k elements are uniformly random from entire slice
///
/// # Arguments
///
/// * `slice` - The slice to partially shuffle
/// * `k` - Number of elements to randomize (clamped to slice length)
/// * `rng` - Random number generator
///
/// # Example
///
/// ```rust
/// use mixed_signals::rng::Rng;
/// use mixed_signals::shuffle::partial_shuffle;
///
/// let mut rng = Rng::with_seed(42);
/// let mut deck: Vec<i32> = (1..=52).collect();
///
/// // Draw 5 cards without shuffling entire deck
/// partial_shuffle(&mut deck, 5, &mut rng);
/// let hand: Vec<i32> = deck[..5].to_vec();
/// ```
///
/// # Edge Cases
///
/// - If `k >= slice.len()`, performs a full shuffle
/// - If `k == 0` or slice is empty, does nothing
pub fn partial_shuffle<T>(slice: &mut [T], k: usize, rng: &mut Rng) {
    let len = slice.len();
    if len <= 1 || k == 0 {
        return;
    }

    let k = k.min(len);

    // Fisher-Yates but only for first k positions
    for i in 0..k {
        let j = (rng.uniform(0.0, (len - i) as f32).floor() as usize).min(len - i - 1) + i;
        slice.swap(i, j);
    }
}

// <FILE>src/shuffle/fnc_partial_shuffle.rs</FILE> - <DESC>Partial shuffle for top-k selection</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
