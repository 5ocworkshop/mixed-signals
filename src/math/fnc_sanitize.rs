// <FILE>mixed-signals/src/math/fnc_sanitize.rs</FILE> - <DESC>Finite-value sanitizers for signal parameters</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Signal generator invalid-input hardening</WCTX>
// <CLOG>Introduce finite-value helpers for NaN/Inf guards</CLOG>

pub(crate) fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

pub(crate) fn finite_or_f64(value: f64, fallback: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

pub(crate) fn finite_or_min(value: f32, min: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.max(min)
    } else {
        fallback
    }
}

pub(crate) fn finite_or_clamp(value: f32, min: f32, max: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}

// <FILE>mixed-signals/src/math/fnc_sanitize.rs</FILE> - <DESC>Finite-value sanitizers for signal parameters</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
