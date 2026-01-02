// <FILE>src/shuffle/fnc_weighted_shuffle.rs</FILE> - <DESC>Priority-biased weighted shuffle algorithm</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Implementing shuffle algorithms phases 2-4</WCTX>
// <CLOG>Initial implementation using Efraimidis-Spirakis algorithm</CLOG>

use crate::rng::Rng;

/// Weighted shuffle using the Efraimidis-Spirakis algorithm.
///
/// Shuffles elements with bias based on weights. Higher weight means the element
/// is more likely to appear earlier in the result. This is ideal for loot tables,
/// priority queues, and sponsored content mixing.
///
/// # Algorithm
///
/// Uses the Efraimidis-Spirakis weighted sampling without replacement:
/// - For each element i, compute `key = uniform(0,1)^(1/weight[i])`
/// - Sort elements by key in descending order
/// - Time complexity: O(n log n)
/// - Space complexity: O(n) for key storage
///
/// # Arguments
///
/// * `slice` - The slice to shuffle in place
/// * `weights` - Weight for each element (higher = more likely to be first)
/// * `rng` - Random number generator for deterministic results
///
/// # Panics
///
/// Does not panic. If weights length differs from slice length, missing weights
/// default to 1.0, extra weights are ignored.
///
/// # Example
///
/// ```rust
/// use mixed_signals::rng::Rng;
/// use mixed_signals::shuffle::weighted_shuffle;
///
/// let mut rng = Rng::with_seed(42);
/// let mut items = vec!["common", "rare", "legendary"];
/// let weights = [10.0, 3.0, 1.0];  // common appears first more often
///
/// weighted_shuffle(&mut items, &weights, &mut rng);
/// ```
///
/// # Weight Handling
///
/// - Zero or negative weights are treated as a small epsilon (0.001)
/// - Non-finite weights default to 1.0
/// - Missing weights default to 1.0
/// - Weights are not normalized; only the relative magnitude matters
pub fn weighted_shuffle<T>(slice: &mut [T], weights: &[f32], rng: &mut Rng) {
    let len = slice.len();
    if len <= 1 {
        return;
    }

    // Compute keys for each element: key = u^(1/weight) where u ~ uniform(0,1)
    // Higher weight -> higher expected key -> earlier position
    let mut keys: Vec<(usize, f32)> = (0..len)
        .map(|i| {
            let mut w = weights.get(i).copied().unwrap_or(1.0);
            if !w.is_finite() {
                w = 1.0;
            }
            let w = w.max(0.001);
            let u = rng.uniform(0.0001, 1.0); // Avoid log(0)
            let key = u.powf(1.0 / w);
            (i, key)
        })
        .collect();

    // Sort by key descending (higher key = earlier position)
    keys.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Apply the permutation using a temporary buffer
    let indices: Vec<usize> = keys.iter().map(|(i, _)| *i).collect();
    apply_permutation(slice, &indices);
}

/// Apply a permutation to a slice in-place using O(n) swaps.
fn apply_permutation<T>(slice: &mut [T], perm: &[usize]) {
    let mut done = vec![false; slice.len()];

    for i in 0..slice.len() {
        if done[i] {
            continue;
        }

        let mut current = i;
        while !done[perm[current]] && perm[current] != i {
            let next = perm[current];
            slice.swap(current, next);
            done[current] = true;
            current = next;
        }
        done[current] = true;
    }
}

// <FILE>src/shuffle/fnc_weighted_shuffle.rs</FILE> - <DESC>Priority-biased weighted shuffle algorithm</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
