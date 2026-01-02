// <FILE>mixed-signals/src/types/signal_or_float.rs</FILE> - <DESC>Parameter that can be static float or signal</DESC>
// <VERS>VERSION: 1.2.0</VERS>
// <WCTX>Migrate time types from f32 to f64</WCTX>
// <CLOG>Updated evaluate methods to use f64 for time parameter</CLOG>

use crate::traits::{Signal, SignalContext};
use crate::types::{SignalBuildError, SignalSpec};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::OnceLock;

/// A parameter that can be either a static float value or a signal specification.
///
/// This enables recipe authors to either specify fixed values or dynamic signal-based
/// values for effect parameters. Deserialization is flexible:
/// - `"5.0"` or `5.0` deserializes as `Static(5.0)`
/// - `{"type": "sine", ...}` deserializes as `Signal(SignalSpec::Sine {...})`
///
/// # Examples
///
/// ```json
/// {"factor": 0.5}  // Static value
/// {"factor": {"type": "sine", "frequency": 2.0, "amplitude": 0.3}}  // Signal-driven
/// ```
#[derive(Serialize, Deserialize)]
#[serde(from = "SignalOrFloatSerde", into = "SignalOrFloatSerde")]
pub enum SignalOrFloat {
    /// A constant static value
    Static(f32),
    /// A dynamic signal specification
    Signal {
        spec: SignalSpec,
        #[serde(skip)]
        cache: OnceLock<Result<Box<dyn Signal>, SignalBuildError>>,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum SignalOrFloatSerde {
    Static(f32),
    Signal(SignalSpec),
}

impl From<SignalOrFloatSerde> for SignalOrFloat {
    fn from(value: SignalOrFloatSerde) -> Self {
        match value {
            SignalOrFloatSerde::Static(v) => SignalOrFloat::Static(v),
            SignalOrFloatSerde::Signal(spec) => SignalOrFloat::from(spec),
        }
    }
}

impl From<SignalOrFloat> for SignalOrFloatSerde {
    fn from(value: SignalOrFloat) -> Self {
        match value {
            SignalOrFloat::Static(v) => SignalOrFloatSerde::Static(v),
            SignalOrFloat::Signal { spec, .. } => SignalOrFloatSerde::Signal(spec),
        }
    }
}

impl fmt::Debug for SignalOrFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignalOrFloat::Static(value) => f.debug_tuple("Static").field(value).finish(),
            SignalOrFloat::Signal { spec, .. } => f.debug_tuple("Signal").field(spec).finish(),
        }
    }
}

impl Clone for SignalOrFloat {
    fn clone(&self) -> Self {
        match self {
            SignalOrFloat::Static(value) => SignalOrFloat::Static(*value),
            SignalOrFloat::Signal { spec, .. } => SignalOrFloat::Signal {
                spec: spec.clone(),
                cache: OnceLock::new(),
            },
        }
    }
}

impl PartialEq for SignalOrFloat {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SignalOrFloat::Static(a), SignalOrFloat::Static(b)) => a == b,
            (SignalOrFloat::Signal { spec: a, .. }, SignalOrFloat::Signal { spec: b, .. }) => {
                a == b
            }
            _ => false,
        }
    }
}

impl SignalOrFloat {
    /// Evaluate the parameter to get a float value at the given time.
    ///
    /// For static values, always returns the same value.
    /// For signal-based values, samples the signal at the given time.
    ///
    /// # Arguments
    /// * `t` - Time parameter for signal evaluation
    /// * `ctx` - Signal context with animation phase and timing info
    ///
    /// # Returns
    /// The evaluated f32 value, or a build error for invalid signal specs
    pub fn evaluate(
        &self,
        t: crate::traits::SignalTime,
        ctx: &SignalContext,
    ) -> Result<f32, SignalBuildError> {
        match self {
            SignalOrFloat::Static(value) => Ok(*value),
            SignalOrFloat::Signal { spec, cache } => {
                let cached = cache.get_or_init(|| spec.build());
                match cached {
                    Ok(signal) => Ok(signal.sample_with_context(t, ctx)),
                    Err(err) => Err(err.clone()),
                }
            }
        }
    }

    /// Evaluate with no context (basic signal sampling).
    ///
    /// Useful for simple cases where you only have a time value.
    /// Equivalent to `evaluate(t, &SignalContext::default())`.
    pub fn evaluate_simple(&self, t: crate::traits::SignalTime) -> Result<f32, SignalBuildError> {
        self.evaluate(t, &SignalContext::default())
    }

    /// Get the static value if this is a Static variant.
    pub fn as_static(&self) -> Option<f32> {
        match self {
            SignalOrFloat::Static(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the signal spec if this is a Signal variant.
    pub fn as_signal(&self) -> Option<&SignalSpec> {
        match self {
            SignalOrFloat::Signal { spec, .. } => Some(spec),
            _ => None,
        }
    }
}

impl From<f32> for SignalOrFloat {
    fn from(value: f32) -> Self {
        SignalOrFloat::Static(value)
    }
}

impl From<SignalSpec> for SignalOrFloat {
    fn from(spec: SignalSpec) -> Self {
        SignalOrFloat::Signal {
            spec,
            cache: OnceLock::new(),
        }
    }
}

impl Default for SignalOrFloat {
    fn default() -> Self {
        SignalOrFloat::Static(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_static_float() {
        let json_val = json!(5.0);
        let param: SignalOrFloat = serde_json::from_value(json_val).unwrap();
        assert_eq!(param, SignalOrFloat::Static(5.0));
    }

    #[test]
    fn deserialize_signal() {
        let json_val = json!({
            "type": "sine",
            "frequency": 2.0,
            "amplitude": 1.0,
            "offset": 0.0,
            "phase": 0.0
        });
        let param: SignalOrFloat = serde_json::from_value(json_val).unwrap();
        assert!(matches!(param, SignalOrFloat::Signal { .. }));
    }

    #[test]
    fn evaluate_static() {
        let param = SignalOrFloat::Static(3.5);
        let value = param.evaluate_simple(0.0).unwrap();
        assert_eq!(value, 3.5);

        let value = param.evaluate_simple(10.0).unwrap();
        assert_eq!(value, 3.5); // Always same value
    }

    #[test]
    fn evaluate_signal() {
        // Constant no longer clamps - returns raw value
        let param = SignalOrFloat::from(SignalSpec::Constant { value: 7.5 });
        let value = param.evaluate_simple(0.0).unwrap();
        assert_eq!(value, 7.5);
    }

    #[test]
    fn from_f32() {
        let param = SignalOrFloat::from(2.5);
        assert_eq!(param, SignalOrFloat::Static(2.5));
    }

    #[test]
    fn as_static() {
        let param = SignalOrFloat::Static(4.0);
        assert_eq!(param.as_static(), Some(4.0));

        let param = SignalOrFloat::from(SignalSpec::Constant { value: 1.0 });
        assert_eq!(param.as_static(), None);
    }

    #[test]
    fn default() {
        let param = SignalOrFloat::default();
        assert_eq!(param, SignalOrFloat::Static(0.0));
    }

    #[test]
    fn evaluate_signal_caches_build() {
        let param = SignalOrFloat::from(SignalSpec::Constant { value: 7.5 });

        if let SignalOrFloat::Signal { cache, .. } = &param {
            assert!(cache.get().is_none());
        }

        let _ = param.evaluate_simple(0.0).unwrap();

        if let SignalOrFloat::Signal { cache, .. } = &param {
            assert!(cache.get().is_some());
        }
    }
}

// <FILE>mixed-signals/src/types/signal_or_float.rs</FILE> - <DESC>Parameter that can be static float or signal</DESC>
// <VERS>END OF VERSION: 1.2.0</VERS>
