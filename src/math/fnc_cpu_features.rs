// <FILE>src/math/fnc_cpu_features.rs</FILE> - <DESC>CPU feature detection for SIMD optimizations</DESC>
// <VERS>VERSION: 1.0.1</VERS>
// <WCTX>Clippy fixes</WCTX>
// <CLOG>Use derive(Default) instead of manual impl</CLOG>

use std::sync::atomic::{AtomicU8, Ordering};

/// CPU feature flags detected at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CpuFeatures {
    pub avx2: bool,
    pub avx: bool,
    pub sse4_2: bool,
    pub fma: bool,
}

// Cached detection result: 0 = not checked, 1 = checked (features stored)
static FEATURES_CHECKED: AtomicU8 = AtomicU8::new(0);
static mut CACHED_FEATURES: CpuFeatures = CpuFeatures {
    avx2: false,
    avx: false,
    sse4_2: false,
    fma: false,
};

/// Detect CPU features at runtime.
///
/// Results are cached after first call for efficiency.
/// Uses `is_x86_feature_detected!` macro on x86_64 targets.
pub fn detect_cpu_features() -> CpuFeatures {
    // Check if we've already detected features
    if FEATURES_CHECKED.load(Ordering::Relaxed) != 0 {
        // SAFETY: Only written once during initialization, then read-only
        return unsafe { CACHED_FEATURES };
    }

    let features = detect_features_impl();

    // Store and mark as checked
    // SAFETY: Only written once, atomic flag ensures single initialization
    unsafe {
        CACHED_FEATURES = features;
    }
    FEATURES_CHECKED.store(1, Ordering::Release);

    features
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
fn detect_features_impl() -> CpuFeatures {
    CpuFeatures {
        avx2: std::arch::is_x86_feature_detected!("avx2"),
        avx: std::arch::is_x86_feature_detected!("avx"),
        sse4_2: std::arch::is_x86_feature_detected!("sse4.2"),
        fma: std::arch::is_x86_feature_detected!("fma"),
    }
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
fn detect_features_impl() -> CpuFeatures {
    CpuFeatures::default()
}

/// Check if AVX2 is available on this CPU.
#[inline]
pub fn has_avx2() -> bool {
    detect_cpu_features().avx2
}

/// Check if FMA (Fused Multiply-Add) is available.
#[inline]
pub fn has_fma() -> bool {
    detect_cpu_features().fma
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cpu_features_cached() {
        let features1 = detect_cpu_features();
        let features2 = detect_cpu_features();
        assert_eq!(features1, features2);
    }

    #[test]
    fn test_has_avx2_returns_bool() {
        // Just verify it returns without panicking
        let _has = has_avx2();
    }

    #[test]
    fn test_detect_features_reasonable() {
        let features = detect_cpu_features();
        // If AVX2 is present, AVX should also be present
        if features.avx2 {
            assert!(features.avx, "AVX2 implies AVX");
        }
        // If AVX is present, SSE4.2 should also be present
        if features.avx {
            assert!(features.sse4_2, "AVX implies SSE4.2");
        }
    }
}

// <FILE>src/math/fnc_cpu_features.rs</FILE> - <DESC>CPU feature detection for SIMD optimizations</DESC>
// <VERS>END OF VERSION: 1.0.1</VERS>
