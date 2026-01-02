// <FILE>src/shuffle/fnc_sattolo.rs</FILE> - <DESC>Sattolo algorithm for cyclic permutations</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Implementing shuffle algorithms for UI and gaming</WCTX>
// <CLOG>Initial implementation</CLOG>

use crate::rng::Rng;

/// Sattolo's algorithm - generates cyclic permutations only.
///
/// Unlike Fisher-Yates, Sattolo guarantees that **no element remains in its
/// original position** (a derangement). Every element moves to a different index.
///
/// # Algorithm
///
/// - Time complexity: O(n)
/// - Space complexity: O(1) - in-place
/// - Generates only cyclic permutations
/// - Guaranteed derangement for n >= 2
///
/// # Use Cases
///
/// - **Secret Santa**: No one draws their own name
/// - **Rotation puzzles**: Every piece must move
/// - **Round-robin with displacement**: Everyone gets someone else's item
///
/// # Example
///
/// ```rust
/// use mixed_signals::rng::Rng;
/// use mixed_signals::shuffle::sattolo;
///
/// let mut rng = Rng::with_seed(42);
/// let original = vec!["Alice", "Bob", "Carol", "Dave"];
/// let mut assignments = original.clone();
///
/// sattolo(&mut assignments, &mut rng);
///
/// // No one has their original position
/// for (i, &name) in assignments.iter().enumerate() {
///     assert_ne!(name, original[i], "{} got themselves!", name);
/// }
/// ```
///
/// # Edge Cases
///
/// - For slices of length 0 or 1, no shuffle is possible (returns unchanged)
/// - For length 2, the only derangement is a swap
pub fn sattolo<T>(slice: &mut [T], rng: &mut Rng) {
    let len = slice.len();
    if len <= 1 {
        return;
    }

    // Sattolo: like Fisher-Yates but j is strictly less than i
    for i in (1..len).rev() {
        // j in [0, i) - never equals i, guaranteeing displacement
        let j = (rng.uniform(0.0, i as f32).floor() as usize).min(i - 1);
        slice.swap(i, j);
    }
}

// <FILE>src/shuffle/fnc_sattolo.rs</FILE> - <DESC>Sattolo algorithm for cyclic permutations</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
