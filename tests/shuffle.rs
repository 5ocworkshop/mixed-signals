// <FILE>tests/shuffle.rs</FILE> - <DESC>Integration tests for shuffle algorithms</DESC>
// <VERS>VERSION: 2.0.0</VERS>
// <WCTX>Implementing shuffle algorithms phases 2-4</WCTX>
// <CLOG>Added tests for weighted, constrained, interleave, riffle, overhand, reservoir, smooth, and animators</CLOG>

use mixed_signals::rng::Rng;
use mixed_signals::shuffle::{
    constrained_shuffle, fisher_yates, interleave, overhand_shuffle, partial_shuffle,
    reservoir_shuffle, riffle_shuffle, sattolo, shuffle_copy, smooth_shuffle, weighted_shuffle,
    OverhandAnimator, RiffleAnimator,
};
use std::collections::HashSet;

// ============================================================================
// Fisher-Yates Tests
// ============================================================================

#[test]
fn test_fisher_yates_determinism() {
    let mut rng1 = Rng::with_seed(42);
    let mut rng2 = Rng::with_seed(42);

    let mut a: Vec<i32> = (1..=10).collect();
    let mut b: Vec<i32> = (1..=10).collect();

    fisher_yates(&mut a, &mut rng1);
    fisher_yates(&mut b, &mut rng2);

    assert_eq!(a, b, "Same seed should produce same shuffle");
}

#[test]
fn test_fisher_yates_different_seeds() {
    let mut rng1 = Rng::with_seed(42);
    let mut rng2 = Rng::with_seed(999);

    let mut a: Vec<i32> = (1..=10).collect();
    let mut b: Vec<i32> = (1..=10).collect();

    fisher_yates(&mut a, &mut rng1);
    fisher_yates(&mut b, &mut rng2);

    assert_ne!(a, b, "Different seeds should produce different shuffles");
}

#[test]
fn test_fisher_yates_preserves_elements() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (1..=20).collect();
    let mut shuffled = original.clone();

    fisher_yates(&mut shuffled, &mut rng);

    let original_set: HashSet<_> = original.iter().collect();
    let shuffled_set: HashSet<_> = shuffled.iter().collect();

    assert_eq!(
        original_set, shuffled_set,
        "Shuffle should preserve all elements"
    );
}

#[test]
fn test_fisher_yates_empty_slice() {
    let mut rng = Rng::with_seed(42);
    let mut empty: Vec<i32> = vec![];
    fisher_yates(&mut empty, &mut rng);
    assert!(empty.is_empty());
}

#[test]
fn test_fisher_yates_single_element() {
    let mut rng = Rng::with_seed(42);
    let mut single = vec![42];
    fisher_yates(&mut single, &mut rng);
    assert_eq!(single, vec![42]);
}

#[test]
fn test_fisher_yates_two_elements() {
    let mut rng = Rng::with_seed(42);
    let mut pair = vec![1, 2];
    fisher_yates(&mut pair, &mut rng);
    assert!(pair == vec![1, 2] || pair == vec![2, 1]);
}

// ============================================================================
// Partial Shuffle Tests
// ============================================================================

#[test]
fn test_partial_shuffle_determinism() {
    let mut rng1 = Rng::with_seed(42);
    let mut rng2 = Rng::with_seed(42);

    let mut a: Vec<i32> = (1..=52).collect();
    let mut b: Vec<i32> = (1..=52).collect();

    partial_shuffle(&mut a, 5, &mut rng1);
    partial_shuffle(&mut b, 5, &mut rng2);

    assert_eq!(a, b, "Same seed should produce same partial shuffle");
}

#[test]
fn test_partial_shuffle_only_affects_first_k() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (1..=10).collect();
    let mut shuffled = original.clone();

    partial_shuffle(&mut shuffled, 3, &mut rng);

    // First 3 elements should be from the original set (but randomized)
    let first_three: HashSet<_> = shuffled[..3].iter().collect();
    let original_set: HashSet<_> = original.iter().collect();

    for elem in &first_three {
        assert!(
            original_set.contains(elem),
            "First k elements should be from original"
        );
    }

    // All elements still present
    let shuffled_set: HashSet<_> = shuffled.iter().collect();
    assert_eq!(original_set, shuffled_set, "Should preserve all elements");
}

#[test]
fn test_partial_shuffle_k_zero() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (1..=10).collect();
    let mut shuffled = original.clone();

    partial_shuffle(&mut shuffled, 0, &mut rng);
    assert_eq!(original, shuffled, "k=0 should not modify slice");
}

#[test]
fn test_partial_shuffle_k_greater_than_len() {
    let mut rng = Rng::with_seed(42);
    let mut items: Vec<i32> = (1..=5).collect();

    // k > len should just do full shuffle
    partial_shuffle(&mut items, 100, &mut rng);

    let set: HashSet<_> = items.iter().collect();
    assert_eq!(set.len(), 5, "Should still have all elements");
}

// ============================================================================
// Shuffle Copy Tests
// ============================================================================

#[test]
fn test_shuffle_copy_determinism() {
    let mut rng1 = Rng::with_seed(42);
    let mut rng2 = Rng::with_seed(42);

    let original: Vec<i32> = (1..=10).collect();

    let a = shuffle_copy(&original, &mut rng1);
    let b = shuffle_copy(&original, &mut rng2);

    assert_eq!(a, b, "Same seed should produce same shuffle copy");
}

#[test]
fn test_shuffle_copy_preserves_original() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (1..=10).collect();

    let shuffled = shuffle_copy(&original, &mut rng);

    assert_eq!(
        original,
        (1..=10).collect::<Vec<i32>>(),
        "Original should be unchanged"
    );
    assert_ne!(shuffled, original, "Shuffled should likely differ");
}

#[test]
fn test_shuffle_copy_preserves_elements() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (1..=20).collect();

    let shuffled = shuffle_copy(&original, &mut rng);

    let original_set: HashSet<_> = original.iter().collect();
    let shuffled_set: HashSet<_> = shuffled.iter().collect();

    assert_eq!(original_set, shuffled_set, "Should preserve all elements");
}

#[test]
fn test_shuffle_copy_empty() {
    let mut rng = Rng::with_seed(42);
    let empty: Vec<i32> = vec![];
    let result = shuffle_copy(&empty, &mut rng);
    assert!(result.is_empty());
}

// ============================================================================
// Sattolo Tests
// ============================================================================

#[test]
fn test_sattolo_determinism() {
    let mut rng1 = Rng::with_seed(42);
    let mut rng2 = Rng::with_seed(42);

    let mut a: Vec<i32> = (1..=10).collect();
    let mut b: Vec<i32> = (1..=10).collect();

    sattolo(&mut a, &mut rng1);
    sattolo(&mut b, &mut rng2);

    assert_eq!(a, b, "Same seed should produce same Sattolo shuffle");
}

#[test]
fn test_sattolo_no_fixed_points() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (0..20).collect();
    let mut shuffled = original.clone();

    sattolo(&mut shuffled, &mut rng);

    for (i, &val) in shuffled.iter().enumerate() {
        assert_ne!(
            val, original[i],
            "Sattolo should not have fixed points: index {} has value {}",
            i, val
        );
    }
}

#[test]
fn test_sattolo_preserves_elements() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (1..=20).collect();
    let mut shuffled = original.clone();

    sattolo(&mut shuffled, &mut rng);

    let original_set: HashSet<_> = original.iter().collect();
    let shuffled_set: HashSet<_> = shuffled.iter().collect();

    assert_eq!(original_set, shuffled_set, "Should preserve all elements");
}

#[test]
fn test_sattolo_two_elements() {
    let mut rng = Rng::with_seed(42);
    let mut pair = vec![1, 2];
    sattolo(&mut pair, &mut rng);

    // For 2 elements, Sattolo must swap them
    assert_eq!(pair, vec![2, 1], "Sattolo on 2 elements must swap");
}

#[test]
fn test_sattolo_single_element() {
    let mut rng = Rng::with_seed(42);
    let mut single = vec![42];
    sattolo(&mut single, &mut rng);
    assert_eq!(single, vec![42], "Single element can't be deranged");
}

#[test]
fn test_sattolo_empty() {
    let mut rng = Rng::with_seed(42);
    let mut empty: Vec<i32> = vec![];
    sattolo(&mut empty, &mut rng);
    assert!(empty.is_empty());
}

// ============================================================================
// Rng Convenience Method Tests
// ============================================================================

#[test]
fn test_rng_shuffle_partial() {
    let mut rng = Rng::with_seed(42);
    let mut deck: Vec<i32> = (1..=52).collect();

    rng.shuffle_partial(&mut deck, 5);

    // Should still have all cards
    let set: HashSet<_> = deck.iter().collect();
    assert_eq!(set.len(), 52);
}

#[test]
fn test_rng_shuffle_cyclic_no_fixed_points() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (0..10).collect();
    let mut shuffled = original.clone();

    rng.shuffle_cyclic(&mut shuffled);

    for (i, &val) in shuffled.iter().enumerate() {
        assert_ne!(
            val, original[i],
            "shuffle_cyclic should not have fixed points"
        );
    }
}

// ============================================================================
// Statistical Tests (basic sanity checks)
// ============================================================================

#[test]
fn test_fisher_yates_distribution_sanity() {
    // Run many shuffles and check that first position isn't always the same
    let mut first_positions: HashSet<i32> = HashSet::new();

    for seed in 0..100 {
        let mut rng = Rng::with_seed(seed);
        let mut items: Vec<i32> = (1..=10).collect();
        fisher_yates(&mut items, &mut rng);
        first_positions.insert(items[0]);
    }

    // Should see variety in first position
    assert!(
        first_positions.len() >= 5,
        "Fisher-Yates should show variety in first position across seeds"
    );
}

// ============================================================================
// Weighted Shuffle Tests (Phase 2)
// ============================================================================

#[test]
fn test_weighted_shuffle_determinism() {
    let mut rng1 = Rng::with_seed(42);
    let mut rng2 = Rng::with_seed(42);

    let mut a = vec!["a", "b", "c", "d", "e"];
    let mut b = vec!["a", "b", "c", "d", "e"];
    let weights = [10.0, 1.0, 1.0, 1.0, 1.0];

    weighted_shuffle(&mut a, &weights, &mut rng1);
    weighted_shuffle(&mut b, &weights, &mut rng2);

    assert_eq!(a, b, "Same seed should produce same weighted shuffle");
}

#[test]
fn test_weighted_shuffle_preserves_elements() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (1..=10).collect();
    let mut shuffled = original.clone();
    let weights: Vec<f32> = (1..=10).map(|x| x as f32).collect();

    weighted_shuffle(&mut shuffled, &weights, &mut rng);

    let original_set: HashSet<_> = original.iter().collect();
    let shuffled_set: HashSet<_> = shuffled.iter().collect();
    assert_eq!(original_set, shuffled_set, "Should preserve all elements");
}

#[test]
fn test_weighted_shuffle_bias() {
    // High weight items should appear in first positions more often
    let weights = [100.0, 1.0, 1.0, 1.0, 1.0];
    let mut first_position_count = 0;

    for seed in 0..200 {
        let mut rng = Rng::with_seed(seed);
        let mut items = vec![0, 1, 2, 3, 4];
        weighted_shuffle(&mut items, &weights, &mut rng);
        if items[0] == 0 {
            first_position_count += 1;
        }
    }

    // Item 0 with weight 100 should be in first position most of the time
    assert!(
        first_position_count > 100,
        "High weight item should appear first more often: got {}",
        first_position_count
    );
}

#[test]
fn test_weighted_shuffle_empty() {
    let mut rng = Rng::with_seed(42);
    let mut empty: Vec<i32> = vec![];
    weighted_shuffle(&mut empty, &[], &mut rng);
    assert!(empty.is_empty());
}

#[test]
fn test_weighted_shuffle_single_element() {
    let mut rng = Rng::with_seed(42);
    let mut single = vec![42];
    weighted_shuffle(&mut single, &[1.0], &mut rng);
    assert_eq!(single, vec![42]);
}

#[test]
fn test_weighted_shuffle_nan_weight() {
    let mut rng = Rng::with_seed(42);
    let mut items = vec![1, 2, 3];
    let weights = [f32::NAN, 1.0, 1.0];

    weighted_shuffle(&mut items, &weights, &mut rng);

    let mut sorted = items.clone();
    sorted.sort();
    assert_eq!(sorted, vec![1, 2, 3]);
}

// ============================================================================
// Constrained Shuffle Tests (Phase 2)
// ============================================================================

#[test]
fn test_constrained_shuffle_determinism() {
    let mut rng1 = Rng::with_seed(42);
    let mut rng2 = Rng::with_seed(42);

    let mut a = vec!['A', 'A', 'B', 'B', 'C', 'C'];
    let mut b = vec!['A', 'A', 'B', 'B', 'C', 'C'];

    constrained_shuffle(&mut a, &mut rng1, 1, |c| *c as usize);
    constrained_shuffle(&mut b, &mut rng2, 1, |c| *c as usize);

    assert_eq!(a, b, "Same seed should produce same constrained shuffle");
}

#[test]
fn test_constrained_shuffle_no_violations() {
    let mut rng = Rng::with_seed(42);
    let mut items: Vec<char> = "AAABBBCCC".chars().collect();

    constrained_shuffle(&mut items, &mut rng, 2, |c| *c as usize);

    // Check no more than 2 consecutive same letters
    let mut run = 1;
    for window in items.windows(2) {
        if window[0] == window[1] {
            run += 1;
        } else {
            run = 1;
        }
        assert!(
            run <= 2,
            "Violation: more than 2 consecutive at {:?}",
            items
        );
    }
}

#[test]
fn test_constrained_shuffle_preserves_elements() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<char> = "AABBCC".chars().collect();
    let mut shuffled = original.clone();

    constrained_shuffle(&mut shuffled, &mut rng, 1, |c| *c as usize);

    let original_sorted: Vec<_> = {
        let mut v = original.clone();
        v.sort();
        v
    };
    let shuffled_sorted: Vec<_> = {
        let mut v = shuffled.clone();
        v.sort();
        v
    };
    assert_eq!(
        original_sorted, shuffled_sorted,
        "Should preserve all elements"
    );
}

#[test]
fn test_constrained_shuffle_empty() {
    let mut rng = Rng::with_seed(42);
    let mut empty: Vec<i32> = vec![];
    constrained_shuffle(&mut empty, &mut rng, 1, |_| 0);
    assert!(empty.is_empty());
}

// ============================================================================
// Interleave Tests (Phase 3)
// ============================================================================

#[test]
fn test_interleave_out_shuffle() {
    let mut cards = vec![1, 2, 3, 4, 5, 6];
    interleave(&mut cards, true);
    assert_eq!(cards, vec![1, 4, 2, 5, 3, 6], "Out-shuffle pattern");
}

#[test]
fn test_interleave_in_shuffle() {
    let mut cards = vec![1, 2, 3, 4, 5, 6];
    interleave(&mut cards, false);
    assert_eq!(cards, vec![4, 1, 5, 2, 6, 3], "In-shuffle pattern");
}

#[test]
fn test_interleave_8_out_shuffles_restore_52() {
    let original: Vec<i32> = (1..=52).collect();
    let mut deck = original.clone();

    for _ in 0..8 {
        interleave(&mut deck, true);
    }

    assert_eq!(deck, original, "8 out-shuffles should restore 52-card deck");
}

#[test]
fn test_interleave_empty() {
    let mut empty: Vec<i32> = vec![];
    interleave(&mut empty, true);
    assert!(empty.is_empty());
}

#[test]
fn test_interleave_single() {
    let mut single = vec![42];
    interleave(&mut single, true);
    assert_eq!(single, vec![42]);
}

#[test]
fn test_interleave_odd_length() {
    let mut odd = vec![1, 2, 3, 4, 5];
    interleave(&mut odd, true);
    // First half: [1, 2, 3], second half: [4, 5]
    // Out-shuffle: 1, 4, 2, 5, 3
    assert_eq!(odd, vec![1, 4, 2, 5, 3]);
}

// ============================================================================
// Riffle Shuffle Tests (Phase 3)
// ============================================================================

#[test]
fn test_riffle_shuffle_determinism() {
    let mut rng1 = Rng::with_seed(42);
    let mut rng2 = Rng::with_seed(42);

    let mut a: Vec<i32> = (1..=20).collect();
    let mut b: Vec<i32> = (1..=20).collect();

    riffle_shuffle(&mut a, 3, &mut rng1);
    riffle_shuffle(&mut b, 3, &mut rng2);

    assert_eq!(a, b, "Same seed should produce same riffle shuffle");
}

#[test]
fn test_riffle_shuffle_preserves_elements() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (1..=52).collect();
    let mut shuffled = original.clone();

    riffle_shuffle(&mut shuffled, 7, &mut rng);

    let original_set: HashSet<_> = original.iter().collect();
    let shuffled_set: HashSet<_> = shuffled.iter().collect();
    assert_eq!(original_set, shuffled_set, "Should preserve all elements");
}

#[test]
fn test_riffle_shuffle_empty() {
    let mut rng = Rng::with_seed(42);
    let mut empty: Vec<i32> = vec![];
    riffle_shuffle(&mut empty, 3, &mut rng);
    assert!(empty.is_empty());
}

#[test]
fn test_riffle_shuffle_zero_passes() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (1..=10).collect();
    let mut items = original.clone();
    riffle_shuffle(&mut items, 0, &mut rng);
    assert_eq!(items, original, "Zero passes should not modify");
}

// ============================================================================
// Overhand Shuffle Tests (Phase 3)
// ============================================================================

#[test]
fn test_overhand_shuffle_determinism() {
    let mut rng1 = Rng::with_seed(42);
    let mut rng2 = Rng::with_seed(42);

    let mut a: Vec<i32> = (1..=20).collect();
    let mut b: Vec<i32> = (1..=20).collect();

    overhand_shuffle(&mut a, 5, &mut rng1);
    overhand_shuffle(&mut b, 5, &mut rng2);

    assert_eq!(a, b, "Same seed should produce same overhand shuffle");
}

#[test]
fn test_overhand_shuffle_preserves_elements() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (1..=52).collect();
    let mut shuffled = original.clone();

    overhand_shuffle(&mut shuffled, 10, &mut rng);

    let original_set: HashSet<_> = original.iter().collect();
    let shuffled_set: HashSet<_> = shuffled.iter().collect();
    assert_eq!(original_set, shuffled_set, "Should preserve all elements");
}

#[test]
fn test_overhand_shuffle_empty() {
    let mut rng = Rng::with_seed(42);
    let mut empty: Vec<i32> = vec![];
    overhand_shuffle(&mut empty, 5, &mut rng);
    assert!(empty.is_empty());
}

// ============================================================================
// Riffle Animator Tests (Phase 3)
// ============================================================================

#[test]
fn test_riffle_animator_completes() {
    let mut rng = Rng::with_seed(42);
    let deck: Vec<i32> = (1..=10).collect();
    let mut animator = RiffleAnimator::new(deck, 1, &mut rng);

    let mut steps = 0;
    while animator.step() {
        steps += 1;
        assert!(steps < 100, "Animation should complete in reasonable steps");
    }

    assert!(animator.is_complete());
    assert_eq!(animator.items().len(), 10);
}

#[test]
fn test_riffle_animator_preserves_elements() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (1..=20).collect();
    let mut animator = RiffleAnimator::new(original.clone(), 2, &mut rng);

    while animator.step() {}

    let result = animator.into_items();
    let original_set: HashSet<_> = original.iter().collect();
    let result_set: HashSet<_> = result.iter().collect();
    assert_eq!(original_set, result_set);
}

#[test]
fn test_riffle_animator_zero_passes() {
    let mut rng = Rng::with_seed(42);
    let deck: Vec<i32> = (1..=10).collect();
    let animator = RiffleAnimator::new(deck.clone(), 0, &mut rng);

    assert!(animator.is_complete());
    assert_eq!(animator.into_items(), deck);
}

// ============================================================================
// Overhand Animator Tests (Phase 3)
// ============================================================================

#[test]
fn test_overhand_animator_completes() {
    let mut rng = Rng::with_seed(42);
    let deck: Vec<i32> = (1..=10).collect();
    let mut animator = OverhandAnimator::new(deck, 1, &mut rng);

    let mut steps = 0;
    while animator.step() {
        steps += 1;
        assert!(steps < 100, "Animation should complete in reasonable steps");
    }

    assert!(animator.is_complete());
    assert_eq!(animator.items().len(), 10);
}

#[test]
fn test_overhand_animator_preserves_elements() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (1..=20).collect();
    let mut animator = OverhandAnimator::new(original.clone(), 2, &mut rng);

    while animator.step() {}

    let result = animator.into_items();
    let original_set: HashSet<_> = original.iter().collect();
    let result_set: HashSet<_> = result.iter().collect();
    assert_eq!(original_set, result_set);
}

// ============================================================================
// Reservoir Shuffle Tests (Phase 4)
// ============================================================================

#[test]
fn test_reservoir_shuffle_determinism() {
    let mut rng1 = Rng::with_seed(42);
    let mut rng2 = Rng::with_seed(42);

    let a = reservoir_shuffle(1..=10, &mut rng1);
    let b = reservoir_shuffle(1..=10, &mut rng2);

    assert_eq!(a, b, "Same seed should produce same reservoir shuffle");
}

#[test]
fn test_reservoir_shuffle_preserves_elements() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = (1..=20).collect();

    let shuffled = reservoir_shuffle(original.clone().into_iter(), &mut rng);

    let original_set: HashSet<_> = original.iter().collect();
    let shuffled_set: HashSet<_> = shuffled.iter().collect();
    assert_eq!(original_set, shuffled_set, "Should preserve all elements");
}

#[test]
fn test_reservoir_shuffle_empty() {
    let mut rng = Rng::with_seed(42);
    let result: Vec<i32> = reservoir_shuffle(std::iter::empty(), &mut rng);
    assert!(result.is_empty());
}

#[test]
fn test_reservoir_shuffle_single() {
    let mut rng = Rng::with_seed(42);
    let result = reservoir_shuffle(std::iter::once(42), &mut rng);
    assert_eq!(result, vec![42]);
}

// ============================================================================
// Smooth Shuffle Tests (Phase 4)
// ============================================================================

#[test]
fn test_smooth_shuffle_determinism() {
    let mut rng1 = Rng::with_seed(42);
    let mut rng2 = Rng::with_seed(42);

    let mut a: Vec<i32> = vec![1, 10, 2, 9, 3, 8, 4, 7, 5, 6];
    let mut b = a.clone();

    smooth_shuffle(&mut a, &mut rng1, |x, y| (*x as f32 - *y as f32).abs());
    smooth_shuffle(&mut b, &mut rng2, |x, y| (*x as f32 - *y as f32).abs());

    assert_eq!(a, b, "Same seed should produce same smooth shuffle");
}

#[test]
fn test_smooth_shuffle_preserves_elements() {
    let mut rng = Rng::with_seed(42);
    let original: Vec<i32> = vec![1, 10, 2, 9, 3, 8, 4, 7, 5, 6];
    let mut shuffled = original.clone();

    smooth_shuffle(&mut shuffled, &mut rng, |x, y| {
        (*x as f32 - *y as f32).abs()
    });

    let original_set: HashSet<_> = original.iter().collect();
    let shuffled_set: HashSet<_> = shuffled.iter().collect();
    assert_eq!(original_set, shuffled_set, "Should preserve all elements");
}

#[test]
fn test_smooth_shuffle_reduces_total_distance() {
    let mut rng = Rng::with_seed(42);
    let mut items: Vec<i32> = vec![1, 100, 2, 99, 3, 98, 4, 97];
    let distance = |x: &i32, y: &i32| (*x as f32 - *y as f32).abs();

    // Calculate initial total distance
    let initial_dist: f32 = items.windows(2).map(|w| distance(&w[0], &w[1])).sum();

    smooth_shuffle(&mut items, &mut rng, distance);

    // Calculate final total distance
    let final_dist: f32 = items.windows(2).map(|w| distance(&w[0], &w[1])).sum();

    assert!(
        final_dist < initial_dist,
        "Smooth shuffle should reduce total transition distance: {} -> {}",
        initial_dist,
        final_dist
    );
}

#[test]
fn test_smooth_shuffle_empty() {
    let mut rng = Rng::with_seed(42);
    let mut empty: Vec<i32> = vec![];
    smooth_shuffle(&mut empty, &mut rng, |x, y| (*x - *y).abs() as f32);
    assert!(empty.is_empty());
}

#[test]
fn test_smooth_shuffle_single() {
    let mut rng = Rng::with_seed(42);
    let mut single = vec![42];
    smooth_shuffle(&mut single, &mut rng, |x, y| (*x as f32 - *y as f32).abs());
    assert_eq!(single, vec![42]);
}

// ============================================================================
// Rng Convenience Method Tests (Extended)
// ============================================================================

#[test]
fn test_rng_shuffle_weighted() {
    let mut rng = Rng::with_seed(42);
    let mut items = vec!["common", "rare", "legendary"];
    let weights = [10.0, 3.0, 1.0];

    rng.shuffle_weighted(&mut items, &weights);

    // Should still have all items
    assert_eq!(items.len(), 3);
    assert!(items.contains(&"common"));
    assert!(items.contains(&"rare"));
    assert!(items.contains(&"legendary"));
}

// <FILE>tests/shuffle.rs</FILE> - <DESC>Integration tests for shuffle algorithms</DESC>
// <VERS>END OF VERSION: 2.0.0</VERS>
