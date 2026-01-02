// <FILE>mixed-signals/src/math/fnc_cubic_bezier.rs</FILE> - <DESC>Cubic Bezier curve solver for easing</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Mixed-signals migration Phase 5 - WP3</WCTX>
// <CLOG>Migrated from tui-geometry EasingCurve - CSS cubic-bezier compatible solver</CLOG>

/// Solve cubic Bézier curve for y given x (time).
///
/// Uses hybrid Newton-Raphson + bisection for robust convergence.
/// Precision target: 1e-6 (suitable for animation).
///
/// Based on standard CSS cubic-bezier implementation with fixed endpoints (0,0) and (1,1).
///
/// # Arguments
///
/// * `t` - Input time value (x-coordinate)
/// * `x1`, `y1` - First control point (must have x1 in [0, 1])
/// * `x2`, `y2` - Second control point (must have x2 in [0, 1])
///
/// # Returns
///
/// The y-coordinate value at the given x (time) value
///
/// # Examples
///
/// ```
/// use mixed_signals::math::solve_bezier;
///
/// // CSS ease: cubic-bezier(0.25, 0.1, 0.25, 1.0)
/// let y = solve_bezier(0.5, 0.25, 0.1, 0.25, 1.0);
/// assert!(y > 0.5); // Accelerates at midpoint
/// ```
pub fn solve_bezier(t: crate::traits::SignalTime, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let x1 = x1.clamp(0.0, 1.0);
    let x2 = x2.clamp(0.0, 1.0);
    // Boundary cases: exact at endpoints
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }

    let t = t as f32;

    // Linear case: no curve
    if (x1 - y1).abs() < 1e-6 && (x2 - y2).abs() < 1e-6 {
        return t;
    }

    // Newton-Raphson to find parameter u such that bezier_x(u) = t
    let mut u = t; // Initial guess
    const MAX_ITERATIONS: usize = 8;
    const EPSILON: f32 = 1e-6;

    for _ in 0..MAX_ITERATIONS {
        let x = bezier_x(u as f64, x1, x2);
        let dx = x - t;

        if dx.abs() < EPSILON {
            // Found solution - return corresponding y value
            return bezier_y(u as f64, y1, y2);
        }

        // Newton-Raphson step: u_new = u - f(u) / f'(u)
        let derivative = bezier_x_derivative(u as f64, x1, x2);

        if derivative.abs() < 1e-9 {
            // Derivative too small - fall back to bisection
            break;
        }

        u -= dx / derivative;

        // Clamp to valid range
        u = u.clamp(0.0, 1.0);
    }

    // Fallback: bisection if Newton-Raphson didn't converge
    let mut low = 0.0_f32;
    let mut high = 1.0_f32;

    for _ in 0..20 {
        // Bisection iterations
        u = (low + high) / 2.0;
        let x = bezier_x(u as f64, x1, x2);

        if (x - t).abs() < EPSILON {
            break;
        }

        if x < t {
            low = u;
        } else {
            high = u;
        }
    }

    bezier_y(u as f64, y1, y2)
}

/// Cubic Bézier x(t) with fixed endpoints (0,0) and (1,1).
#[inline]
pub fn bezier_x(t: crate::traits::SignalTime, x1: f32, x2: f32) -> f32 {
    // Cubic Bézier formula: B(t) = (1-t)³P0 + 3(1-t)²t*P1 + 3(1-t)t²P2 + t³P3
    // With P0 = (0, 0) and P3 = (1, 1):
    let t = t.clamp(0.0, 1.0) as f32;
    let t2 = t * t;
    let t3 = t2 * t;
    let one_minus_t = 1.0 - t;
    let one_minus_t2 = one_minus_t * one_minus_t;

    3.0 * one_minus_t2 * t * x1 + 3.0 * one_minus_t * t2 * x2 + t3
}

/// Cubic Bézier y(t) with fixed endpoints (0,0) and (1,1).
#[inline]
pub fn bezier_y(t: crate::traits::SignalTime, y1: f32, y2: f32) -> f32 {
    let t = t.clamp(0.0, 1.0) as f32;
    let t2 = t * t;
    let t3 = t2 * t;
    let one_minus_t = 1.0 - t;
    let one_minus_t2 = one_minus_t * one_minus_t;

    3.0 * one_minus_t2 * t * y1 + 3.0 * one_minus_t * t2 * y2 + t3
}

/// Derivative of bezier_x with respect to t.
#[inline]
pub fn bezier_x_derivative(t: crate::traits::SignalTime, x1: f32, x2: f32) -> f32 {
    // d/dt of cubic Bézier x(t)
    let t = t.clamp(0.0, 1.0) as f32;
    let one_minus_t = 1.0 - t;
    3.0 * one_minus_t * one_minus_t * x1
        + 6.0 * one_minus_t * t * (x2 - x1)
        + 3.0 * t * t * (1.0 - x2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamps_control_points() {
        let cases = [0.2, 0.5, 0.8];
        for t in cases {
            let unclamped = solve_bezier(t, -10.0, 0.1, 10.0, 1.0);
            let clamped = solve_bezier(t, 0.0, 0.1, 1.0, 1.0);
            assert!((unclamped - clamped).abs() < 1e-6);
        }
    }
}

// <FILE>mixed-signals/src/math/fnc_cubic_bezier.rs</FILE> - <DESC>Cubic Bezier curve solver for easing</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
