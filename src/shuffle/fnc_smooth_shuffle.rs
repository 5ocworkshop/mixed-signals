// <FILE>src/shuffle/fnc_smooth_shuffle.rs</FILE> - <DESC>Transition-optimized smooth shuffle</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Implementing shuffle algorithms phases 2-4</WCTX>
// <CLOG>Initial implementation using greedy nearest-neighbor with randomization</CLOG>

use crate::rng::Rng;
use crate::shuffle::fisher_yates;

/// Smooth shuffle that minimizes transition jarring.
///
/// Reorders elements to minimize the sum of "distances" between consecutive
/// items, where distance is defined by the provided function. Useful for
/// creating smooth playlists, color sequences, or image galleries.
///
/// # Algorithm
///
/// Uses greedy nearest-neighbor with random perturbation:
/// 1. Start from a random element
/// 2. Greedily select the nearest unvisited element
/// 3. Apply random swaps to escape local minima
///
/// - Time complexity: O(n²)
/// - Space complexity: O(n)
///
/// # Arguments
///
/// * `slice` - The slice to shuffle in place
/// * `rng` - Random number generator
/// * `distance` - Function returning distance between two elements (higher = more jarring)
///
/// # Distance Function
///
/// The distance function should return a non-negative f32:
/// - 0.0 = perfectly smooth transition
/// - Higher values = more jarring transition
///
/// # Example
///
/// ```rust
/// use mixed_signals::rng::Rng;
/// use mixed_signals::shuffle::smooth_shuffle;
///
/// let mut rng = Rng::with_seed(42);
/// let mut colors = vec![10, 50, 20, 90, 30, 80, 40];
///
/// // Minimize jumps between color values
/// smooth_shuffle(&mut colors, &mut rng, |a, b| (*a as f32 - *b as f32).abs());
///
/// // Colors are now ordered to minimize adjacent differences
/// ```
///
/// # Use Cases
///
/// - Music playlists: Smooth BPM/key transitions
/// - Color palettes: Minimize perceptual jumps
/// - Image galleries: Group similar images
/// - Data visualization: Smooth axis ordering
pub fn smooth_shuffle<T, F>(slice: &mut [T], rng: &mut Rng, distance: F)
where
    F: Fn(&T, &T) -> f32,
{
    let len = slice.len();
    if len <= 2 {
        return;
    }

    // Start with a random shuffle to avoid bias
    fisher_yates(slice, rng);

    // Greedy nearest-neighbor construction
    let mut used = vec![false; len];
    let mut result_order: Vec<usize> = Vec::with_capacity(len);

    // Start from a random element
    let start = (rng.uniform(0.0, len as f32).floor() as usize).min(len - 1);
    result_order.push(start);
    used[start] = true;

    // Greedily add nearest neighbors
    for _ in 1..len {
        let last = result_order[result_order.len() - 1];
        let mut best_idx = 0;
        let mut best_dist = f32::MAX;

        for (i, is_used) in used.iter().enumerate() {
            if *is_used {
                continue;
            }
            let d = distance(&slice[last], &slice[i]);
            if d < best_dist {
                best_dist = d;
                best_idx = i;
            }
        }

        result_order.push(best_idx);
        used[best_idx] = true;
    }

    // Apply some random swaps to escape local minima (2-opt style)
    let swap_attempts = len / 2;
    for _ in 0..swap_attempts {
        let i = (rng.uniform(0.0, (len - 1) as f32).floor() as usize).min(len - 2);
        let j = (rng.uniform((i + 1) as f32, len as f32).floor() as usize).min(len - 1);

        // Calculate cost change of reversing segment [i+1, j]
        let cost_before = edge_cost(&result_order, i, slice, &distance)
            + edge_cost(&result_order, j, slice, &distance);

        // Try swap
        result_order[i + 1..=j].reverse();

        let cost_after = edge_cost(&result_order, i, slice, &distance)
            + edge_cost(&result_order, j, slice, &distance);

        // Keep if better, or sometimes accept worse (simulated annealing)
        if cost_after > cost_before && rng.uniform(0.0, 1.0) > 0.3 {
            // Revert
            result_order[i + 1..=j].reverse();
        }
    }

    // Apply permutation
    apply_permutation(slice, &result_order);
}

/// Calculate edge cost at position i (between i and i+1).
fn edge_cost<T, F>(order: &[usize], i: usize, slice: &[T], distance: &F) -> f32
where
    F: Fn(&T, &T) -> f32,
{
    if i + 1 >= order.len() {
        return 0.0;
    }
    distance(&slice[order[i]], &slice[order[i + 1]])
}

/// Apply a permutation to a slice in-place.
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

// <FILE>src/shuffle/fnc_smooth_shuffle.rs</FILE> - <DESC>Transition-optimized smooth shuffle</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
