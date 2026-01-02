// <FILE>mixed-signals/src/random/fnc_hash_to_index.rs</FILE> - <DESC>Deterministic hash-to-index mapping for character selection</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>feat-20251224-170136: Complete signal-driven content effects</WCTX>
// <CLOG>Created public hash_to_index utility to replace fnc_random.rs RNG usage</CLOG>

/// Hash function for deterministic pseudo-random values.
///
/// Uses SplitMix64 algorithm for fast, high-quality hashing.
fn hash_u64_pair(a: u64, b: u64) -> u64 {
    let mut x = a.wrapping_add(b).wrapping_mul(0x517cc1b727220a95);
    x ^= x >> 32;
    x = x.wrapping_mul(0x9e3779b97f4a7c15);
    x ^= x >> 32;
    x
}

/// Maps a pair of u64 seeds to an index in the range [0, len).
///
/// Provides deterministic random index selection without requiring RNG state.
/// Same inputs always produce the same output index.
///
/// # Arguments
/// * `seed_a` - First component of the seed (e.g., base seed)
/// * `seed_b` - Second component (e.g., character index + progress)
/// * `len` - Length of the array/collection to index into
///
/// # Returns
/// Index in the range [0, len)
///
/// # Example
/// ```
/// use mixed_signals::random::hash_to_index;
///
/// let base_seed = 42u64;
/// let char_index = 5u64;
/// let charset_len = 26; // a-z
///
/// let idx = hash_to_index(base_seed, char_index, charset_len);
/// assert!(idx < charset_len);
///
/// // Deterministic: same inputs = same output
/// assert_eq!(idx, hash_to_index(base_seed, char_index, charset_len));
/// ```
pub fn hash_to_index(seed_a: u64, seed_b: u64, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let hash = hash_u64_pair(seed_a, seed_b);
    (hash % len as u64) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_to_index_determinism() {
        let idx1 = hash_to_index(42, 100, 26);
        let idx2 = hash_to_index(42, 100, 26);
        assert_eq!(idx1, idx2, "Same inputs should produce same index");
    }

    #[test]
    fn test_hash_to_index_range() {
        for len in [1, 10, 26, 100, 256] {
            for i in 0..100 {
                let idx = hash_to_index(42, i, len);
                assert!(idx < len, "Index {} out of range [0, {})", idx, len);
            }
        }
    }

    #[test]
    fn test_hash_to_index_different_seeds() {
        let idx1 = hash_to_index(1, 100, 26);
        let idx2 = hash_to_index(2, 100, 26);
        // Different seeds should (usually) produce different indices
        // Not guaranteed but very likely for good hash function
        assert_ne!(idx1, idx2);
    }

    #[test]
    fn test_hash_to_index_zero_len() {
        let idx = hash_to_index(42, 100, 0);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_hash_to_index_distribution() {
        // Rough distribution test: indices should spread across range
        let mut counts = vec![0; 10];
        for i in 0..1000 {
            let idx = hash_to_index(42, i, 10);
            counts[idx] += 1;
        }
        // Each bucket should have roughly 100 ± 50 items
        for count in counts {
            assert!(
                count > 50 && count < 150,
                "Poor distribution: bucket has {}",
                count
            );
        }
    }
}

// <FILE>mixed-signals/src/random/fnc_hash_to_index.rs</FILE> - <DESC>Deterministic hash-to-index mapping for character selection</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
