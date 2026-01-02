// <FILE>src/math/fnc_fast_random_batch.rs</FILE> - <DESC>Batch random generation with SIMD when available</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>RNG performance optimization</WCTX>
// <CLOG>Initial creation - AVX2 batch random with scalar fallback</CLOG>

use super::fnc_fast_random::fast_random;

/// Generate a batch of random f32 values.
///
/// When AVX2 is available, processes 8 values at a time for ~4x throughput.
/// Falls back to scalar fast_random when AVX2 is not available.
///
/// # Arguments
/// * `seed` - Base seed for determinism
/// * `start_input` - Starting input value (each output uses start_input + index)
/// * `output` - Slice to fill with random values in 0.0..1.0
///
/// # Example
/// ```
/// use mixed_signals::math::fast_random_batch;
/// let mut buf = [0.0f32; 16];
/// fast_random_batch(42, 0, &mut buf);
/// assert!(buf.iter().all(|&v| v >= 0.0 && v < 1.0));
/// ```
#[inline]
pub fn fast_random_batch(seed: u64, start_input: u64, output: &mut [f32]) {
    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    {
        if super::fnc_cpu_features::has_avx2() {
            // SAFETY: AVX2 is confirmed available
            unsafe { fast_random_batch_avx2(seed, start_input, output) };
            return;
        }
    }

    // Scalar fallback
    fast_random_batch_scalar(seed, start_input, output);
}

/// Scalar implementation - always available
#[inline]
fn fast_random_batch_scalar(seed: u64, start_input: u64, output: &mut [f32]) {
    for (i, slot) in output.iter_mut().enumerate() {
        *slot = fast_random(seed, start_input.wrapping_add(i as u64));
    }
}

/// AVX2 implementation - processes 4 u64s at a time
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[target_feature(enable = "avx2")]
unsafe fn fast_random_batch_avx2(seed: u64, start_input: u64, output: &mut [f32]) {
    use std::arch::x86_64::*;

    let len = output.len();
    let chunks = len / 4;

    // SplitMix64 constants as vectors
    let mul1 = _mm256_set1_epi64x(0x9e3779b97f4a7c15_u64 as i64);
    let mul2 = _mm256_set1_epi64x(0xbf58476d1ce4e5b9_u64 as i64);
    let mul3 = _mm256_set1_epi64x(0x94d049bb133111eb_u64 as i64);
    let seed_vec = _mm256_set1_epi64x(seed as i64);
    let scale = 1.0f32 / ((1u64 << 24) as f32);

    for chunk in 0..chunks {
        let base = start_input.wrapping_add((chunk * 4) as u64);
        let inputs = _mm256_set_epi64x(
            (base + 3) as i64,
            (base + 2) as i64,
            (base + 1) as i64,
            base as i64,
        );

        // h = seed + input
        let mut h = _mm256_add_epi64(seed_vec, inputs);

        // h *= 0x9e3779b97f4a7c15 (wrapping)
        h = avx2_mul64(h, mul1);

        // h = (h ^ (h >> 30)) * 0xbf58476d1ce4e5b9
        let h_shr30 = _mm256_srli_epi64(h, 30);
        h = _mm256_xor_si256(h, h_shr30);
        h = avx2_mul64(h, mul2);

        // h = (h ^ (h >> 27)) * 0x94d049bb133111eb
        let h_shr27 = _mm256_srli_epi64(h, 27);
        h = _mm256_xor_si256(h, h_shr27);
        h = avx2_mul64(h, mul3);

        // h ^= h >> 31
        let h_shr31 = _mm256_srli_epi64(h, 31);
        h = _mm256_xor_si256(h, h_shr31);

        // Extract top 24 bits and convert to float
        let h_shr40 = _mm256_srli_epi64(h, 40);

        // Extract 4 u64 values and convert to f32
        let mut vals = [0u64; 4];
        _mm256_storeu_si256(vals.as_mut_ptr() as *mut __m256i, h_shr40);

        let out_idx = chunk * 4;
        output[out_idx] = vals[0] as f32 * scale;
        output[out_idx + 1] = vals[1] as f32 * scale;
        output[out_idx + 2] = vals[2] as f32 * scale;
        output[out_idx + 3] = vals[3] as f32 * scale;
    }

    // Handle remaining elements with scalar
    let remainder_start = chunks * 4;
    for i in remainder_start..len {
        output[i] = fast_random(seed, start_input.wrapping_add(i as u64));
    }
}

/// AVX2 64-bit multiply (emulated since AVX2 doesn't have native 64-bit mul)
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn avx2_mul64(
    a: std::arch::x86_64::__m256i,
    b: std::arch::x86_64::__m256i,
) -> std::arch::x86_64::__m256i {
    use std::arch::x86_64::*;

    // 64-bit multiply using 32-bit parts:
    // (a_lo + a_hi * 2^32) * (b_lo + b_hi * 2^32)
    // = a_lo * b_lo + (a_lo * b_hi + a_hi * b_lo) * 2^32
    // We only need the low 64 bits, so drop a_hi * b_hi * 2^64

    // Get low 32 bits of each 64-bit lane
    let a_lo = a;
    let b_lo = b;

    // Get high 32 bits shifted to low position
    let a_hi = _mm256_srli_epi64(a, 32);
    let b_hi = _mm256_srli_epi64(b, 32);

    // Low * low (64-bit result)
    let lo_lo = _mm256_mul_epu32(a_lo, b_lo);

    // Cross products (only need low 32 bits of result, shifted left 32)
    let lo_hi = _mm256_mul_epu32(a_lo, b_hi);
    let hi_lo = _mm256_mul_epu32(a_hi, b_lo);

    // Combine: lo_lo + (lo_hi + hi_lo) << 32
    let cross = _mm256_add_epi64(lo_hi, hi_lo);
    let cross_shifted = _mm256_slli_epi64(cross, 32);

    _mm256_add_epi64(lo_lo, cross_shifted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_matches_scalar() {
        let mut batch_out = [0.0f32; 16];
        fast_random_batch(42, 100, &mut batch_out);

        // Compare with scalar version
        for (i, &val) in batch_out.iter().enumerate() {
            let scalar = fast_random(42, 100 + i as u64);
            assert!(
                (val - scalar).abs() < 1e-6,
                "Mismatch at index {}: batch={}, scalar={}",
                i,
                val,
                scalar
            );
        }
    }

    #[test]
    fn test_batch_range() {
        let mut out = [0.0f32; 100];
        fast_random_batch(42, 0, &mut out);
        for (i, &v) in out.iter().enumerate() {
            assert!(
                (0.0..1.0).contains(&v),
                "Value {} at index {} out of range",
                v,
                i
            );
        }
    }

    #[test]
    fn test_batch_deterministic() {
        let mut a = [0.0f32; 8];
        let mut b = [0.0f32; 8];
        fast_random_batch(42, 50, &mut a);
        fast_random_batch(42, 50, &mut b);
        assert_eq!(a, b);
    }

    #[test]
    fn test_batch_odd_sizes() {
        // Test sizes that aren't multiples of 4/8
        for size in [1, 2, 3, 5, 7, 9, 13, 17] {
            let mut out = vec![0.0f32; size];
            fast_random_batch(42, 0, &mut out);
            for &v in &out {
                assert!((0.0..1.0).contains(&v));
            }
        }
    }
}

// <FILE>src/math/fnc_fast_random_batch.rs</FILE> - <DESC>Batch random generation with SIMD when available</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
