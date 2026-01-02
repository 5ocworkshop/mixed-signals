// <FILE>src/shuffle/fnc_interleave.rs</FILE> - <DESC>Deterministic Faro (perfect interleave) shuffle</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Implementing shuffle algorithms phases 2-4</WCTX>
// <CLOG>Initial implementation of deterministic Faro shuffle</CLOG>

/// Perfect interleave (Faro shuffle).
///
/// A deterministic shuffle that perfectly interleaves two halves of the deck.
/// **No randomness involved** - this is a fixed permutation.
///
/// # Algorithm
///
/// - Splits the slice at the midpoint
/// - Interleaves cards from each half alternately
/// - Time complexity: O(n)
/// - Space complexity: O(n) for temporary buffer
///
/// # Arguments
///
/// * `slice` - The slice to shuffle in place
/// * `out_shuffle` - If true, first element stays first (out-shuffle).
///   If false, first element moves to second position (in-shuffle).
///
/// # Cycle Property
///
/// For a 52-card deck:
/// - 8 out-shuffles restore original order
/// - 52 in-shuffles restore original order
///
/// # Example
///
/// ```rust
/// use mixed_signals::shuffle::interleave;
///
/// let mut cards = vec![1, 2, 3, 4, 5, 6];
/// interleave(&mut cards, true);  // out-shuffle
/// assert_eq!(cards, vec![1, 4, 2, 5, 3, 6]);
/// ```
///
/// # Use Cases
///
/// - Magic tricks (predictable transformations)
/// - Merge-sort style animations
/// - Tutorials demonstrating shuffling
/// - Generating specific permutation sequences
pub fn interleave<T>(slice: &mut [T], out_shuffle: bool)
where
    T: Clone,
{
    let len = slice.len();
    if len <= 1 {
        return;
    }

    // Split at midpoint (first half gets extra card if odd)
    let mid = len.div_ceil(2);
    let (left, right) = slice.split_at(mid);

    // Build interleaved result
    let mut result: Vec<T> = Vec::with_capacity(len);

    let mut l_iter = left.iter();
    let mut r_iter = right.iter();

    if out_shuffle {
        // Out-shuffle: L R L R L R... (first card stays first)
        for l in l_iter.by_ref() {
            result.push(l.clone());
            if let Some(r) = r_iter.next() {
                result.push(r.clone());
            }
        }
    } else {
        // In-shuffle: R L R L R L... (first card moves to second)
        loop {
            if let Some(r) = r_iter.next() {
                result.push(r.clone());
            }
            if let Some(l) = l_iter.next() {
                result.push(l.clone());
            } else {
                break;
            }
        }
    }

    // Copy back to slice
    slice.clone_from_slice(&result);
}

// <FILE>src/shuffle/fnc_interleave.rs</FILE> - <DESC>Deterministic Faro (perfect interleave) shuffle</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
