// <FILE>mixed-signals/src/math/fnc_quadratic_bezier.rs</FILE> - <DESC>Quadratic Bezier curve formula</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Mixed-signals migration Phase 5 - WP3</WCTX>
// <CLOG>Migrated from tui-geometry - pure quadratic Bezier formula</CLOG>

/// Calculates a point on a quadratic Bézier curve.
///
/// Uses the formula: B(t) = (1-t)²·P₀ + 2(1-t)t·P₁ + t²·P₂
///
/// # Arguments
///
/// * `t` - Parameter in range [0, 1] (0 = start, 1 = end)
/// * `p0` - Start point value
/// * `p1` - Control point value
/// * `p2` - End point value
///
/// # Examples
///
/// ```
/// use mixed_signals::math::quadratic_bezier;
///
/// // Linear interpolation (control point at midpoint)
/// let value = quadratic_bezier(0.5, 0.0, 0.5, 1.0);
/// assert!((value - 0.5).abs() < 0.001);
/// ```
pub fn quadratic_bezier(t: crate::traits::SignalTime, p0: f32, p1: f32, p2: f32) -> f32 {
    let t = t.clamp(0.0, 1.0) as f32;
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    (uu * p0) + (2.0 * u * t * p1) + (tt * p2)
}

// <FILE>mixed-signals/src/math/fnc_quadratic_bezier.rs</FILE> - <DESC>Quadratic Bezier curve formula</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
