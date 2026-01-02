// <FILE>src/shuffle/fnc_fisher_yates.rs</FILE> - <DESC>Fisher-Yates shuffle algorithm</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Implementing shuffle algorithms for UI and gaming</WCTX>
// <CLOG>Initial implementation</CLOG>

use crate::rng::Rng;

/// Fisher-Yates shuffle (Knuth shuffle).
///
/// The gold standard shuffle algorithm. Produces an unbiased permutation where
/// every possible ordering is equally likely.
///
/// # Algorithm
///
/// - Time complexity: O(n)
/// - Space complexity: O(1) - in-place
/// - Unbiased: Every permutation equally probable
///
/// # Example
///
/// ```rust
/// use mixed_signals::rng::Rng;
/// use mixed_signals::shuffle::fisher_yates;
///
/// let mut rng = Rng::with_seed(42);
/// let mut items = vec![1, 2, 3, 4, 5];
/// fisher_yates(&mut items, &mut rng);
/// // items is now shuffled deterministically
/// ```
///
/// # Determinism
///
/// Same seed produces same shuffle:
///
/// ```rust
/// use mixed_signals::rng::Rng;
/// use mixed_signals::shuffle::fisher_yates;
///
/// let mut rng1 = Rng::with_seed(123);
/// let mut rng2 = Rng::with_seed(123);
/// let mut a = vec![1, 2, 3, 4, 5];
/// let mut b = vec![1, 2, 3, 4, 5];
///
/// fisher_yates(&mut a, &mut rng1);
/// fisher_yates(&mut b, &mut rng2);
/// assert_eq!(a, b);
/// ```
pub fn fisher_yates<T>(slice: &mut [T], rng: &mut Rng) {
    for i in (1..slice.len()).rev() {
        let j = (rng.uniform(0.0, (i + 1) as f32).floor() as usize).min(i);
        slice.swap(i, j);
    }
}

// <FILE>src/shuffle/fnc_fisher_yates.rs</FILE> - <DESC>Fisher-Yates shuffle algorithm</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
