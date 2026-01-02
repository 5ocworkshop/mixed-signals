// <FILE>src/math/fnc_derive_seed.rs</FILE> - <DESC>Deterministic seed derivation for ChaCha8Rng</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Consolidating duplicated derive_seed functions</WCTX>
// <CLOG>Initial creation - extracted from 8 duplicate implementations</CLOG>

//! Deterministic seed derivation for reproducible randomness.
//!
//! This function combines a base seed with an input value to produce a 32-byte
//! seed suitable for ChaCha8Rng. The mixing function ensures good distribution
//! even with sequential inputs.

/// Derive a 32-byte seed from a base seed and input value.
///
/// Uses a mixing function to combine the inputs into a deterministic seed
/// suitable for `ChaCha8Rng::from_seed()`. The same (base_seed, input) pair
/// always produces the same output.
///
/// # Arguments
///
/// * `base_seed` - Primary seed value (e.g., from SignalContext or user config)
/// * `input` - Secondary value to mix in (e.g., frame number, octave index)
///
/// # Example (internal use only)
///
/// ```ignore
/// // This function is pub(crate), used internally:
/// let seed_bytes = derive_seed(42, 100);
/// let mut rng = ChaCha8Rng::from_seed(seed_bytes);
/// ```
#[inline]
pub fn derive_seed(base_seed: u64, input: u64) -> [u8; 32] {
    let combined = base_seed
        .wrapping_add(input)
        .wrapping_mul(0x517cc1b727220a95);
    let mut seed_bytes = [0u8; 32];
    for (i, byte) in seed_bytes.iter_mut().enumerate() {
        *byte = (combined.wrapping_shr((i * 8) as u32) & 0xFF) as u8;
    }
    seed_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_seed_deterministic() {
        let s1 = derive_seed(42, 100);
        let s2 = derive_seed(42, 100);
        assert_eq!(s1, s2, "Same inputs must produce same seed");
    }

    #[test]
    fn test_derive_seed_different_base() {
        let s1 = derive_seed(1, 100);
        let s2 = derive_seed(2, 100);
        assert_ne!(
            s1, s2,
            "Different base seeds must produce different results"
        );
    }

    #[test]
    fn test_derive_seed_different_input() {
        let s1 = derive_seed(42, 1);
        let s2 = derive_seed(42, 2);
        assert_ne!(s1, s2, "Different inputs must produce different results");
    }

    #[test]
    fn test_derive_seed_sequential_inputs_differ() {
        // Verify sequential inputs produce sufficiently different seeds
        let seeds: Vec<[u8; 32]> = (0..10).map(|i| derive_seed(42, i)).collect();
        for i in 0..seeds.len() {
            for j in (i + 1)..seeds.len() {
                assert_ne!(
                    seeds[i], seeds[j],
                    "Sequential inputs {} and {} should differ",
                    i, j
                );
            }
        }
    }

    #[test]
    fn test_derive_seed_edge_cases() {
        // Should handle edge values without panic
        let _ = derive_seed(0, 0);
        let _ = derive_seed(u64::MAX, u64::MAX);
        let _ = derive_seed(u64::MAX, 0);
        let _ = derive_seed(0, u64::MAX);
    }
}

// <FILE>src/math/fnc_derive_seed.rs</FILE> - <DESC>Deterministic seed derivation for ChaCha8Rng</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
