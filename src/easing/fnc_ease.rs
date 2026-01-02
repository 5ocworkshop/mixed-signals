// <FILE>mixed-signals/src/easing/fnc_ease.rs</FILE> - <DESC>Easing functions</DESC>
// <VERS>VERSION: 1.0.0</VERS>
// <WCTX>Mixed-signals migration Phase 5 - WP2</WCTX>
// <CLOG>Migrated from tui-geometry - 27 easing variants (Linear through CircInOut)</CLOG>

use crate::traits::SignalTime;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EasingType {
    #[default]
    #[serde(alias = "Linear")]
    Linear,
    #[serde(alias = "QuadIn")]
    QuadIn,
    #[serde(alias = "QuadOut")]
    QuadOut,
    #[serde(alias = "QuadInOut")]
    QuadInOut,
    #[serde(alias = "CubicIn")]
    CubicIn,
    #[serde(alias = "CubicOut")]
    CubicOut,
    #[serde(alias = "CubicInOut")]
    CubicInOut,
    #[serde(alias = "SineIn")]
    SineIn,
    #[serde(alias = "SineOut")]
    SineOut,
    #[serde(alias = "SineInOut")]
    SineInOut,
    #[serde(alias = "BackIn")]
    BackIn,
    #[serde(alias = "BackOut")]
    BackOut,
    #[serde(alias = "BackInOut")]
    BackInOut,
    #[serde(alias = "ElasticIn")]
    ElasticIn,
    #[serde(alias = "ElasticOut")]
    ElasticOut,
    #[serde(alias = "ElasticInOut")]
    ElasticInOut,
    #[serde(alias = "BounceIn")]
    BounceIn,
    #[serde(alias = "BounceOut")]
    BounceOut,
    #[serde(alias = "BounceInOut")]
    BounceInOut,
    #[serde(alias = "ExpoIn")]
    ExpoIn,
    #[serde(alias = "ExpoOut")]
    ExpoOut,
    #[serde(alias = "ExpoInOut")]
    ExpoInOut,
    #[serde(alias = "CircIn")]
    CircIn,
    #[serde(alias = "CircOut")]
    CircOut,
    #[serde(alias = "CircInOut")]
    CircInOut,
}

/// Calculates the eased value for time `t` (usually 0.0 to 1.0).
pub fn ease(t: SignalTime, type_: EasingType) -> f32 {
    let t = t.clamp(0.0, 1.0) as f32;
    match type_ {
        EasingType::Linear => t,
        EasingType::QuadIn => t * t,
        EasingType::QuadOut => -t * (t - 2.0),
        EasingType::QuadInOut => {
            let t = t * 2.0;
            if t < 1.0 {
                0.5 * t * t
            } else {
                let t = t - 1.0;
                -0.5 * (t * (t - 2.0) - 1.0)
            }
        }
        EasingType::CubicIn => t * t * t,
        EasingType::CubicOut => {
            let t = t - 1.0;
            t * t * t + 1.0
        }
        EasingType::CubicInOut => {
            let t = t * 2.0;
            if t < 1.0 {
                0.5 * t * t * t
            } else {
                let t = t - 2.0;
                0.5 * (t * t * t + 2.0)
            }
        }
        EasingType::SineIn => 1.0 - (t * PI / 2.0).cos(),
        EasingType::SineOut => (t * PI / 2.0).sin(),
        EasingType::SineInOut => -((PI * t).cos() - 1.0) / 2.0,
        EasingType::BackIn => {
            const C1: f32 = 1.70158;
            const C3: f32 = C1 + 1.0;
            C3 * t * t * t - C1 * t * t
        }
        EasingType::BackOut => {
            const C1: f32 = 1.70158;
            const C3: f32 = C1 + 1.0;
            let t = t - 1.0;
            1.0 + C3 * t * t * t + C1 * t * t
        }
        EasingType::BackInOut => {
            const C1: f32 = 1.70158;
            const C2: f32 = C1 * 1.525;
            let t = t * 2.0;
            if t < 1.0 {
                (t * t * ((C2 + 1.0) * t - C2)) / 2.0
            } else {
                let t = t - 2.0;
                (t * t * ((C2 + 1.0) * t + C2) + 2.0) / 2.0
            }
        }
        EasingType::ElasticIn => {
            if t == 0.0 || t == 1.0 {
                t
            } else {
                let p = 0.3;
                let s = p / 4.0;
                let t = t - 1.0;
                -(2.0_f32.powf(10.0 * t) * ((t - s) * (2.0 * PI) / p).sin())
            }
        }
        EasingType::ElasticOut => {
            if t == 0.0 || t == 1.0 {
                t
            } else {
                let p = 0.3;
                let s = p / 4.0;
                2.0_f32.powf(-10.0 * t) * ((t - s) * (2.0 * PI) / p).sin() + 1.0
            }
        }
        EasingType::ElasticInOut => {
            if t == 0.0 || t == 1.0 {
                t
            } else {
                let p = 0.3 * 1.5;
                let s = p / 4.0;
                let t = t * 2.0;
                if t < 1.0 {
                    let t = t - 1.0;
                    -0.5 * (2.0_f32.powf(10.0 * t) * ((t - s) * (2.0 * PI) / p).sin())
                } else {
                    let t = t - 1.0;
                    2.0_f32.powf(-10.0 * t) * ((t - s) * (2.0 * PI) / p).sin() * 0.5 + 1.0
                }
            }
        }
        EasingType::BounceIn => 1.0 - bounce_out(1.0 - t),
        EasingType::BounceOut => bounce_out(t),
        EasingType::BounceInOut => {
            if t < 0.5 {
                (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0
            } else {
                (1.0 + bounce_out(2.0 * t - 1.0)) / 2.0
            }
        }
        EasingType::ExpoIn => {
            if t == 0.0 {
                0.0
            } else {
                2.0_f32.powf(10.0 * t - 10.0)
            }
        }
        EasingType::ExpoOut => {
            if t == 1.0 {
                1.0
            } else {
                1.0 - 2.0_f32.powf(-10.0 * t)
            }
        }
        EasingType::ExpoInOut => {
            if t == 0.0 || t == 1.0 {
                t
            } else if t < 0.5 {
                2.0_f32.powf(20.0 * t - 10.0) / 2.0
            } else {
                (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
            }
        }
        EasingType::CircIn => 1.0 - (1.0 - t * t).sqrt(),
        EasingType::CircOut => {
            let t = t - 1.0;
            (1.0 - t * t).sqrt()
        }
        EasingType::CircInOut => {
            if t < 0.5 {
                let t = 2.0 * t;
                (1.0 - (1.0 - t * t).sqrt()) / 2.0
            } else {
                let t = -2.0 * t + 2.0;
                ((1.0 - t * t).sqrt() + 1.0) / 2.0
            }
        }
    }
}

/// Helper for bounce easing calculations.
fn bounce_out(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;

    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t = t - 1.5 / D1;
        N1 * t * t + 0.75
    } else if t < 2.5 / D1 {
        let t = t - 2.25 / D1;
        N1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / D1;
        N1 * t * t + 0.984375
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ease_clamps_out_of_range_linear() {
        assert!((ease(1.5, EasingType::Linear) - 1.0).abs() < 0.001);
        assert!((ease(-0.5, EasingType::Linear) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_ease_clamps_to_avoid_nan() {
        let v = ease(1.5, EasingType::CircIn);
        assert!(v.is_finite());
    }
}

// <FILE>mixed-signals/src/easing/fnc_ease.rs</FILE> - <DESC>Easing functions</DESC>
// <VERS>END OF VERSION: 1.0.0</VERS>
