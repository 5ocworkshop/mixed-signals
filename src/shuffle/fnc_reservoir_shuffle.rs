// <FILE>src/shuffle/fnc_reservoir_shuffle.rs</FILE> - <DESC>Streaming reservoir shuffle for iterators</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Implementing shuffle algorithms phases 2-4</WCTX>
// <CLOG>Initial implementation of reservoir shuffle</CLOG>

use crate::rng::Rng;

/// Reservoir shuffle for streaming/iterator input.
///
/// Shuffles elements from an iterator without knowing the total count upfront.
/// Produces a uniformly random permutation of all elements.
///
/// # Algorithm
///
/// Uses inside-out Fisher-Yates on a growing reservoir:
/// 1. For element at index i, pick random j in [0, i]
/// 2. Place new element at position j, moving displaced element to end
/// 3. Result is uniformly shuffled
///
/// - Time complexity: O(n)
/// - Space complexity: O(n) for result storage
///
/// # Arguments
///
/// * `iter` - Iterator providing elements to shuffle
/// * `rng` - Random number generator
///
/// # Returns
///
/// A `Vec<T>` containing all elements in shuffled order.
///
/// # Example
///
/// ```rust
/// use mixed_signals::rng::Rng;
/// use mixed_signals::shuffle::reservoir_shuffle;
///
/// let mut rng = Rng::with_seed(42);
///
/// // Shuffle from an iterator
/// let numbers = 1..=10;
/// let shuffled = reservoir_shuffle(numbers, &mut rng);
/// assert_eq!(shuffled.len(), 10);
/// ```
///
/// # Use Cases
///
/// - Shuffling streaming data
/// - Unknown collection sizes (files, network streams)
/// - Memory-efficient shuffle of iterator sources
/// - Lazy evaluation with random ordering
pub fn reservoir_shuffle<T, I>(iter: I, rng: &mut Rng) -> Vec<T>
where
    I: Iterator<Item = T>,
{
    let mut result: Vec<T> = Vec::new();

    for (i, item) in iter.enumerate() {
        // Pick random position in [0, i] (inclusive of current position)
        let j = (rng.uniform(0.0, (i + 1) as f32).floor() as usize).min(i);

        if j < result.len() {
            // Swap: new item goes to position j, old item goes to end
            result.push(item);
            let last = result.len() - 1;
            result.swap(j, last);
        } else {
            // j == result.len(), just append
            result.push(item);
        }
    }

    result
}

// <FILE>src/shuffle/fnc_reservoir_shuffle.rs</FILE> - <DESC>Streaming reservoir shuffle for iterators</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
