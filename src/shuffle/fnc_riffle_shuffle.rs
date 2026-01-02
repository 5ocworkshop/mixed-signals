// <FILE>src/shuffle/fnc_riffle_shuffle.rs</FILE> - <DESC>Gilbert-Shannon-Reeds riffle shuffle</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Implementing shuffle algorithms phases 2-4</WCTX>
// <CLOG>Initial implementation of GSR riffle shuffle model</CLOG>

use crate::rng::Rng;

/// Riffle shuffle using the Gilbert-Shannon-Reeds (GSR) model.
///
/// Simulates realistic card riffle shuffling where the deck is cut near the
/// middle and cards are dropped alternately from each half with probability
/// proportional to the remaining cards in that half.
///
/// # Algorithm
///
/// For each pass:
/// 1. Cut deck at random point near middle (uniform in middle 50%)
/// 2. Drop cards from halves with probability = remaining_in_half / total_remaining
/// 3. Repeat for specified number of passes
///
/// - Time complexity: O(n) per pass
/// - Space complexity: O(n) for temporary buffer
///
/// # Arguments
///
/// * `slice` - The slice to shuffle in place
/// * `passes` - Number of riffle passes (7 recommended for near-uniform)
/// * `rng` - Random number generator
///
/// # Mixing Quality
///
/// - 1 pass: Very predictable
/// - 3 passes: Moderate mixing
/// - 7 passes: Near-uniform distribution (Bayer-Diaconis result)
/// - 10+ passes: Diminishing returns
///
/// # Example
///
/// ```rust
/// use mixed_signals::rng::Rng;
/// use mixed_signals::shuffle::riffle_shuffle;
///
/// let mut rng = Rng::with_seed(42);
/// let mut deck: Vec<i32> = (1..=52).collect();
///
/// riffle_shuffle(&mut deck, 7, &mut rng);
/// // deck is now shuffled with realistic riffle mechanics
/// ```
pub fn riffle_shuffle<T: Clone>(slice: &mut [T], passes: usize, rng: &mut Rng) {
    let len = slice.len();
    if len <= 1 || passes == 0 {
        return;
    }

    for _ in 0..passes {
        riffle_once(slice, rng);
    }
}

/// Perform a single riffle shuffle pass.
fn riffle_once<T: Clone>(slice: &mut [T], rng: &mut Rng) {
    let len = slice.len();
    if len <= 1 {
        return;
    }

    // Cut point: uniform random in middle 50% of deck
    let cut_min = (len / 4).max(1);
    let cut_max = (3 * len / 4).min(len - 1);
    let cut = (rng.uniform(cut_min as f32, cut_max as f32).floor() as usize).clamp(1, len - 1);

    let left: Vec<T> = slice[..cut].to_vec();
    let right: Vec<T> = slice[cut..].to_vec();

    let mut result: Vec<T> = Vec::with_capacity(len);
    let mut l_idx = 0;
    let mut r_idx = 0;
    let l_len = left.len();
    let r_len = right.len();

    // GSR model: drop from each half with probability proportional to remaining
    while l_idx < l_len || r_idx < r_len {
        let l_remaining = l_len - l_idx;
        let r_remaining = r_len - r_idx;
        let total = l_remaining + r_remaining;

        if total == 0 {
            break;
        }

        // Probability of dropping from left
        let p_left = l_remaining as f32 / total as f32;

        if rng.uniform(0.0, 1.0) < p_left && l_idx < l_len {
            result.push(left[l_idx].clone());
            l_idx += 1;
        } else if r_idx < r_len {
            result.push(right[r_idx].clone());
            r_idx += 1;
        } else if l_idx < l_len {
            result.push(left[l_idx].clone());
            l_idx += 1;
        }
    }

    slice.clone_from_slice(&result);
}

// <FILE>src/shuffle/fnc_riffle_shuffle.rs</FILE> - <DESC>Gilbert-Shannon-Reeds riffle shuffle</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
