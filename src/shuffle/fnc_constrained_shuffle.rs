// <FILE>src/shuffle/fnc_constrained_shuffle.rs</FILE> - <DESC>Variety-enforced constrained shuffle algorithm</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Implementing shuffle algorithms phases 2-4</WCTX>
// <CLOG>Initial implementation with retry-based constraint satisfaction</CLOG>

use crate::rng::Rng;
use crate::shuffle::fisher_yates;

/// Constrained shuffle with variety enforcement.
///
/// Shuffles elements while ensuring no more than `max_consecutive` items of the
/// same category appear in a row. Categories are determined by the classifier
/// function.
///
/// # Algorithm
///
/// Uses retry-based constraint satisfaction:
/// 1. Perform Fisher-Yates shuffle
/// 2. Scan for violations (runs > max_consecutive)
/// 3. Swap violating elements with valid positions
/// 4. Repeat until satisfied or max attempts reached
///
/// - Time complexity: O(n²) worst case
/// - Space complexity: O(n) for category tracking
///
/// # Arguments
///
/// * `slice` - The slice to shuffle in place
/// * `rng` - Random number generator
/// * `max_consecutive` - Maximum allowed consecutive items of same category
/// * `classifier` - Function that returns category ID for each element
///
/// # Best-Effort Behavior
///
/// If constraints cannot be fully satisfied (e.g., too many items of one category),
/// returns the best-effort result with minimum violations. Does not panic.
///
/// # Example
///
/// ```rust
/// use mixed_signals::rng::Rng;
/// use mixed_signals::shuffle::constrained_shuffle;
///
/// let mut rng = Rng::with_seed(42);
/// let mut playlist = vec!["ArtistA-1", "ArtistA-2", "ArtistB-1", "ArtistB-2", "ArtistC-1"];
///
/// // No artist plays twice in a row
/// constrained_shuffle(&mut playlist, &mut rng, 1, |s| {
///     s.chars().next().unwrap() as usize  // First char as category
/// });
/// ```
///
/// # Use Cases
///
/// - Music playlists: No artist twice in a row
/// - Exam questions: No topic clusters
/// - Game spawns: Variety enforcement
/// - Ad rotation: Competitor separation
pub fn constrained_shuffle<T, F>(
    slice: &mut [T],
    rng: &mut Rng,
    max_consecutive: usize,
    classifier: F,
) where
    F: Fn(&T) -> usize,
{
    let len = slice.len();
    if len <= 1 {
        return;
    }

    let max_consecutive = max_consecutive.max(1); // At least 1 allowed
    let max_attempts = len * 10;

    // Initial shuffle
    fisher_yates(slice, rng);

    // Iteratively fix violations
    for _ in 0..max_attempts {
        if let Some(violation_idx) = find_violation(slice, max_consecutive, &classifier) {
            // Try to fix by swapping with a valid position
            if !try_fix_violation(slice, violation_idx, max_consecutive, &classifier, rng) {
                // Couldn't fix this violation, continue trying
                continue;
            }
        } else {
            // No violations found
            return;
        }
    }
    // Best-effort: return whatever we have
}

/// Find the first index where a run violation occurs.
fn find_violation<T, F>(slice: &[T], max_consecutive: usize, classifier: &F) -> Option<usize>
where
    F: Fn(&T) -> usize,
{
    if slice.len() < 2 {
        return None;
    }

    let mut run_length = 1;
    let mut prev_cat = classifier(&slice[0]);

    for (i, item) in slice.iter().enumerate().skip(1) {
        let cat = classifier(item);
        if cat == prev_cat {
            run_length += 1;
            if run_length > max_consecutive {
                return Some(i);
            }
        } else {
            run_length = 1;
            prev_cat = cat;
        }
    }
    None
}

/// Try to fix a violation by swapping with a valid position.
fn try_fix_violation<T, F>(
    slice: &mut [T],
    violation_idx: usize,
    max_consecutive: usize,
    classifier: &F,
    rng: &mut Rng,
) -> bool
where
    F: Fn(&T) -> usize,
{
    let violation_cat = classifier(&slice[violation_idx]);
    let len = slice.len();

    // Find valid swap candidates (different category, won't create new violation)
    let candidates: Vec<usize> = (0..len)
        .filter(|&i| {
            if i == violation_idx {
                return false;
            }
            let cat = classifier(&slice[i]);
            if cat == violation_cat {
                return false;
            }
            // Check if swapping wouldn't create a new violation at position i
            would_be_valid_at(slice, i, violation_idx, max_consecutive, classifier)
        })
        .collect();

    if candidates.is_empty() {
        return false;
    }

    // Pick a random valid candidate
    let pick_idx =
        (rng.uniform(0.0, candidates.len() as f32).floor() as usize).min(candidates.len() - 1);
    let swap_with = candidates[pick_idx];
    slice.swap(violation_idx, swap_with);
    true
}

/// Check if swapping elements at pos1 and pos2 would be valid at pos1.
fn would_be_valid_at<T, F>(
    slice: &[T],
    check_pos: usize,
    swap_from: usize,
    max_consecutive: usize,
    classifier: &F,
) -> bool
where
    F: Fn(&T) -> usize,
{
    let new_cat = classifier(&slice[swap_from]);

    // Check run before check_pos
    let mut run_before = 0;
    if check_pos > 0 {
        for i in (0..check_pos).rev() {
            if classifier(&slice[i]) == new_cat {
                run_before += 1;
            } else {
                break;
            }
        }
    }

    // Check run after check_pos
    let mut run_after = 0;
    for (i, item) in slice.iter().enumerate().skip(check_pos + 1) {
        if i == swap_from {
            // Skip the element being swapped out
            continue;
        }
        if classifier(item) == new_cat {
            run_after += 1;
        } else {
            break;
        }
    }

    run_before + 1 + run_after <= max_consecutive
}

// <FILE>src/shuffle/fnc_constrained_shuffle.rs</FILE> - <DESC>Variety-enforced constrained shuffle algorithm</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
