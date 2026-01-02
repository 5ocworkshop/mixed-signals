// <FILE>src/shuffle/fnc_shuffle_copy.rs</FILE> - <DESC>Non-mutating shuffle returning new Vec</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Implementing shuffle algorithms for UI and gaming</WCTX>
// <CLOG>Initial implementation</CLOG>

use crate::rng::Rng;

/// Non-mutating shuffle - returns a shuffled copy.
///
/// Creates a new `Vec` containing all elements from the input slice in
/// shuffled order, leaving the original unchanged. Uses the inside-out
/// variant of Fisher-Yates.
///
/// # Algorithm
///
/// - Time complexity: O(n)
/// - Space complexity: O(n) - creates new Vec
/// - Unbiased: Every permutation equally probable
///
/// # Example
///
/// ```rust
/// use mixed_signals::rng::Rng;
/// use mixed_signals::shuffle::shuffle_copy;
///
/// let mut rng = Rng::with_seed(42);
/// let original = vec![1, 2, 3, 4, 5];
/// let shuffled = shuffle_copy(&original, &mut rng);
///
/// // Original unchanged
/// assert_eq!(original, vec![1, 2, 3, 4, 5]);
/// // Shuffled is a permutation
/// assert_ne!(shuffled, original); // Very likely different
/// ```
///
/// # Use Cases
///
/// - Immutable data patterns
/// - When you need both original and shuffled order
/// - Functional programming style
pub fn shuffle_copy<T: Clone>(slice: &[T], rng: &mut Rng) -> Vec<T> {
    if slice.is_empty() {
        return Vec::new();
    }

    // Inside-out Fisher-Yates
    let mut result = Vec::with_capacity(slice.len());

    for (i, item) in slice.iter().enumerate() {
        let j = (rng.uniform(0.0, (i + 1) as f32).floor() as usize).min(i);
        if j == i {
            result.push(item.clone());
        } else {
            result.push(result[j].clone());
            result[j] = item.clone();
        }
    }

    result
}

// <FILE>src/shuffle/fnc_shuffle_copy.rs</FILE> - <DESC>Non-mutating shuffle returning new Vec</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
