// <FILE>mixed-signals/src/types/signal_spec.rs</FILE> - <DESC>SignalSpec enum for serialization</DESC>
// <VERS>VERSION: 2.1.0</VERS>
// <WCTX>Deduplication refactor</WCTX>
// <CLOG>Deprecated Sum/Scale variants (now build to Add/Multiply)

use crate::composition::{Add, FrequencyMod, Mix, Multiply, VcaCentered};
use crate::envelopes::{Adsr, Impact, LinearEnvelope};
use crate::generators::{
    Constant, Keyframes, PhaseAccumulator, PhaseSine, Pulse, Ramp, Sawtooth, Sine, Square, Step,
    Triangle,
};
use crate::noise::{PerlinNoise, WhiteNoise};
use crate::processing::{Abs, Clamp, Invert, Quantize, Remap};
use crate::random::{
    CorrelatedNoise, GaussianNoise, ImpulseNoise, PerCharacterNoise, PinkNoise, PoissonNoise,
    SeededRandom, SpatialNoise, StudentTNoise,
};
use crate::traits::Signal;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SignalBuildError {
    Gaussian(String),
    Poisson(String),
    Correlated(String),
    StudentT(String),
}

impl fmt::Display for SignalBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignalBuildError::Gaussian(msg) => write!(f, "GaussianNoise build failed: {}", msg),
            SignalBuildError::Poisson(msg) => write!(f, "PoissonNoise build failed: {}", msg),
            SignalBuildError::Correlated(msg) => {
                write!(f, "CorrelatedNoise build failed: {}", msg)
            }
            SignalBuildError::StudentT(msg) => write!(f, "StudentTNoise build failed: {}", msg),
        }
    }
}

impl std::error::Error for SignalBuildError {}

/// Serializable specification for any signal type.
///
/// This enum allows signals to be defined in JSON configuration files
/// and composed recursively.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignalSpec {
    // === Oscillators ===
    Sine {
        #[serde(default = "default_frequency")]
        frequency: f32,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default)]
        offset: f32,
        #[serde(default)]
        phase: f32,
    },
    Triangle {
        #[serde(default = "default_frequency")]
        frequency: f32,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default)]
        offset: f32,
        #[serde(default)]
        phase: f32,
    },
    Square {
        #[serde(default = "default_frequency")]
        frequency: f32,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default)]
        offset: f32,
        #[serde(default)]
        phase: f32,
        #[serde(default = "default_duty")]
        duty: f32,
    },
    Sawtooth {
        #[serde(default = "default_frequency")]
        frequency: f32,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default)]
        offset: f32,
        #[serde(default)]
        phase: f32,
        #[serde(default)]
        inverted: bool,
    },

    // === Utility ===
    Constant {
        value: f32,
    },
    Ramp {
        #[serde(default)]
        start: f32,
        #[serde(default = "default_one")]
        end: f32,
        #[serde(default = "default_one")]
        duration: f32,
    },
    Step {
        #[serde(default)]
        before: f32,
        #[serde(default = "default_one")]
        after: f32,
        #[serde(default = "default_half")]
        threshold: f32,
    },
    Pulse {
        #[serde(default)]
        low: f32,
        #[serde(default = "default_one")]
        high: f32,
        start: f32,
        end: f32,
    },

    // === Noise ===
    WhiteNoise {
        #[serde(default)]
        seed: u64,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default = "default_sample_rate")]
        sample_rate: f32,
    },
    Perlin {
        #[serde(default)]
        seed: u64,
        #[serde(default = "default_one")]
        scale: f32,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default = "default_octaves")]
        octaves: u8,
        #[serde(default = "default_persistence")]
        persistence: f32,
    },

    // === Random/RNG ===
    SeededRandom {
        #[serde(default)]
        seed: u64,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default)]
        offset: f32,
    },
    SpatialNoise {
        #[serde(default)]
        seed: u64,
        #[serde(default = "default_one")]
        frequency: f32,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
    },
    GaussianNoise {
        #[serde(default)]
        seed: u64,
        #[serde(default = "default_gaussian_std_dev")]
        std_dev: f32,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default)]
        offset: f32,
    },
    PoissonNoise {
        #[serde(default)]
        seed: u64,
        #[serde(default = "default_lambda")]
        lambda: f32,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default)]
        offset: f32,
    },
    CorrelatedNoise {
        #[serde(default)]
        seed: u64,
        #[serde(default = "default_correlation")]
        correlation: f32,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default)]
        offset: f32,
    },
    PinkNoise {
        #[serde(default)]
        seed: u64,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default)]
        offset: f32,
    },
    PerCharacterNoise {
        #[serde(default)]
        base_seed: u64,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default)]
        offset: f32,
    },
    StudentTNoise {
        #[serde(default)]
        seed: u64,
        #[serde(default = "default_degrees_of_freedom")]
        degrees_of_freedom: f32,
        #[serde(default = "default_student_t_scale")]
        scale: f32,
        #[serde(default = "default_amplitude")]
        amplitude: f32,
        #[serde(default)]
        offset: f32,
    },
    ImpulseNoise {
        #[serde(default)]
        seed: u64,
        #[serde(default = "default_impulse_rate")]
        rate_hz: f32,
        #[serde(default = "default_impulse_width")]
        impulse_width: f32,
    },

    // === Envelopes ===
    Adsr {
        #[serde(default = "default_attack")]
        attack: f32,
        #[serde(default = "default_decay")]
        decay: f32,
        #[serde(default = "default_sustain")]
        sustain: f32,
        #[serde(default = "default_release")]
        release: f32,
        #[serde(default = "default_one")]
        peak: f32,
    },
    Impact {
        #[serde(default = "default_one")]
        intensity: f32,
        #[serde(default = "default_decay_rate")]
        decay: f32,
    },
    LinearEnvelope {
        #[serde(default = "default_attack")]
        attack: f32,
        #[serde(default = "default_release")]
        release: f32,
        #[serde(default = "default_one")]
        peak: f32,
    },

    // === Composition ===
    Add {
        a: Box<SignalSpec>,
        b: Box<SignalSpec>,
    },
    Multiply {
        a: Box<SignalSpec>,
        b: Box<SignalSpec>,
    },
    /// Deprecated: identical to Multiply. Kept for config file backward compatibility.
    #[deprecated(since = "2.0.0", note = "Use Multiply instead")]
    Scale {
        a: Box<SignalSpec>,
        b: Box<SignalSpec>,
    },
    /// Deprecated: identical to Add. Kept for config file backward compatibility.
    #[deprecated(since = "2.0.0", note = "Use Add instead")]
    Sum {
        a: Box<SignalSpec>,
        b: Box<SignalSpec>,
    },
    Mix {
        a: Box<SignalSpec>,
        b: Box<SignalSpec>,
        #[serde(default = "default_half")]
        mix: f32,
    },
    FrequencyMod {
        carrier: Box<SignalSpec>,
        modulator: Box<SignalSpec>,
        #[serde(default = "default_one")]
        depth: f32,
        #[serde(default = "default_frequency")]
        carrier_freq: f32,
    },
    VcaCentered {
        carrier: Box<SignalSpec>,
        amplitude: Box<SignalSpec>,
    },
    PhaseAccumulator {
        frequency: Box<SignalSpec>,
        #[serde(default)]
        initial_phase: f32,
    },
    /// Convert phase [0,1) to sine [-1,1]. Final stage of true FM synthesis.
    PhaseSine {
        phase: Box<SignalSpec>,
    },
    /// Piecewise linear interpolation between keyframe points.
    /// Enables arbitrary curve shapes that can't be expressed with oscillators.
    Keyframes {
        /// List of (time, value) keyframe pairs
        keyframes: Vec<(f32, f32)>,
    },

    // === Processing ===
    Clamp {
        signal: Box<SignalSpec>,
        #[serde(default = "default_zero")]
        min: f32,
        #[serde(default = "default_one")]
        max: f32,
    },
    Quantize {
        signal: Box<SignalSpec>,
        #[serde(default = "default_levels")]
        levels: u8,
    },
    Remap {
        signal: Box<SignalSpec>,
        #[serde(default = "default_zero")]
        in_min: f32,
        #[serde(default = "default_one")]
        in_max: f32,
        #[serde(default)]
        out_min: f32,
        #[serde(default = "default_one")]
        out_max: f32,
    },
    Invert {
        signal: Box<SignalSpec>,
    },
    Abs {
        signal: Box<SignalSpec>,
    },
}

// Default value functions
fn default_frequency() -> f32 {
    1.0
}
fn default_amplitude() -> f32 {
    1.0
}
fn default_duty() -> f32 {
    0.5
}
fn default_one() -> f32 {
    1.0
}
fn default_half() -> f32 {
    0.5
}
fn default_zero() -> f32 {
    0.0
}
fn default_sample_rate() -> f32 {
    60.0
}
fn default_octaves() -> u8 {
    1
}
fn default_persistence() -> f32 {
    0.5
}
fn default_attack() -> f32 {
    0.1
}
fn default_decay() -> f32 {
    0.1
}
fn default_sustain() -> f32 {
    0.7
}
fn default_release() -> f32 {
    0.2
}
fn default_decay_rate() -> f32 {
    3.0
}
fn default_levels() -> u8 {
    4
}
fn default_lambda() -> f32 {
    2.0
}
fn default_correlation() -> f32 {
    0.95
}
fn default_gaussian_std_dev() -> f32 {
    1.0
}
fn default_degrees_of_freedom() -> f32 {
    3.0
}
fn default_student_t_scale() -> f32 {
    1.0
}
fn default_impulse_rate() -> f32 {
    10.0
}
fn default_impulse_width() -> f32 {
    0.001
}

impl SignalSpec {
    /// Build a boxed Signal from this specification.
    pub fn build(&self) -> Result<Box<dyn Signal>, SignalBuildError> {
        match self {
            // Oscillators
            SignalSpec::Sine {
                frequency,
                amplitude,
                offset,
                phase,
            } => Ok(Box::new(Sine::new(*frequency, *amplitude, *offset, *phase))),

            SignalSpec::Triangle {
                frequency,
                amplitude,
                offset,
                phase,
            } => Ok(Box::new(Triangle::new(
                *frequency, *amplitude, *offset, *phase,
            ))),

            SignalSpec::Square {
                frequency,
                amplitude,
                offset,
                phase,
                duty,
            } => Ok(Box::new(Square::new(
                *frequency, *amplitude, *offset, *phase, *duty,
            ))),

            SignalSpec::Sawtooth {
                frequency,
                amplitude,
                offset,
                phase,
                inverted,
            } => Ok(Box::new(Sawtooth::new(
                *frequency, *amplitude, *offset, *phase, *inverted,
            ))),

            // Utility
            SignalSpec::Constant { value } => Ok(Box::new(Constant::new(*value))),

            SignalSpec::Ramp {
                start,
                end,
                duration,
            } => Ok(Box::new(Ramp::new(*start, *end, *duration))),

            SignalSpec::Step {
                before,
                after,
                threshold,
            } => Ok(Box::new(Step::new(*before, *after, *threshold))),

            SignalSpec::Pulse {
                low,
                high,
                start,
                end,
            } => Ok(Box::new(Pulse::new(*low, *high, *start, *end))),

            // Noise
            SignalSpec::WhiteNoise {
                seed,
                amplitude,
                sample_rate,
            } => Ok(Box::new(WhiteNoise::new(*seed, *amplitude, *sample_rate))),

            SignalSpec::Perlin {
                seed,
                scale,
                amplitude,
                octaves,
                persistence,
            } => Ok(Box::new(
                PerlinNoise::new(*seed, *scale, *amplitude).with_octaves(*octaves, *persistence),
            )),

            // Random/RNG
            SignalSpec::SeededRandom {
                seed,
                amplitude,
                offset,
            } => Ok(Box::new(SeededRandom::new(*seed, *amplitude, *offset))),

            SignalSpec::SpatialNoise {
                seed,
                frequency,
                amplitude,
            } => Ok(Box::new(SpatialNoise::new(*seed, *frequency, *amplitude))),

            SignalSpec::GaussianNoise {
                seed,
                std_dev,
                amplitude,
                offset,
            } => GaussianNoise::new(*seed, *std_dev, *amplitude, *offset)
                .map(|noise| Box::new(noise) as Box<dyn Signal>)
                .map_err(SignalBuildError::Gaussian),

            SignalSpec::PoissonNoise {
                seed,
                lambda,
                amplitude,
                offset,
            } => PoissonNoise::new(*seed, *lambda, *amplitude, *offset)
                .map(|noise| Box::new(noise) as Box<dyn Signal>)
                .map_err(SignalBuildError::Poisson),

            SignalSpec::CorrelatedNoise {
                seed,
                correlation,
                amplitude,
                offset,
            } => CorrelatedNoise::new(*seed, *correlation, *amplitude, *offset)
                .map(|noise| Box::new(noise) as Box<dyn Signal>)
                .map_err(SignalBuildError::Correlated),

            SignalSpec::PinkNoise {
                seed,
                amplitude,
                offset,
            } => Ok(Box::new(PinkNoise::new(*seed, *amplitude, *offset))),

            SignalSpec::PerCharacterNoise {
                base_seed,
                amplitude,
                offset,
            } => Ok(Box::new(PerCharacterNoise::new(
                *base_seed, *amplitude, *offset,
            ))),

            SignalSpec::StudentTNoise {
                seed,
                degrees_of_freedom,
                scale,
                amplitude,
                offset,
            } => StudentTNoise::new(*degrees_of_freedom, *seed, *scale, *amplitude, *offset)
                .map(|noise| Box::new(noise) as Box<dyn Signal>)
                .map_err(SignalBuildError::StudentT),

            SignalSpec::ImpulseNoise {
                seed,
                rate_hz,
                impulse_width,
            } => Ok(Box::new(ImpulseNoise::with_width(
                *rate_hz,
                *seed,
                *impulse_width,
            ))),

            // Envelopes
            SignalSpec::Adsr {
                attack,
                decay,
                sustain,
                release,
                peak,
            } => Ok(Box::new(
                Adsr::new(*attack, *decay, *sustain, *release).with_peak(*peak),
            )),

            SignalSpec::Impact { intensity, decay } => {
                Ok(Box::new(Impact::new(*intensity, *decay)))
            }

            SignalSpec::LinearEnvelope {
                attack,
                release,
                peak,
            } => Ok(Box::new(
                LinearEnvelope::new(*attack, *release).with_peak(*peak),
            )),

            // Composition
            SignalSpec::Add { a, b } => Ok(Box::new(Add::new(a.build()?, b.build()?))),

            SignalSpec::Multiply { a, b } => Ok(Box::new(Multiply::new(a.build()?, b.build()?))),

            #[allow(deprecated)]
            SignalSpec::Scale { a, b } => Ok(Box::new(Multiply::new(a.build()?, b.build()?))),

            #[allow(deprecated)]
            SignalSpec::Sum { a, b } => Ok(Box::new(Add::new(a.build()?, b.build()?))),

            SignalSpec::Mix { a, b, mix } => Ok(Box::new(Mix::new(a.build()?, b.build()?, *mix))),

            SignalSpec::FrequencyMod {
                carrier,
                modulator,
                depth,
                carrier_freq,
            } => Ok(Box::new(FrequencyMod::new(
                carrier.build()?,
                modulator.build()?,
                *depth,
                *carrier_freq,
            ))),

            SignalSpec::VcaCentered { carrier, amplitude } => Ok(Box::new(VcaCentered::new(
                carrier.build()?,
                amplitude.build()?,
            ))),

            SignalSpec::PhaseAccumulator {
                frequency,
                initial_phase,
            } => Ok(Box::new(PhaseAccumulator::new(
                frequency.build()?,
                *initial_phase,
            ))),
            SignalSpec::PhaseSine { phase } => Ok(Box::new(PhaseSine::new(phase.build()?))),

            SignalSpec::Keyframes { keyframes } => Ok(Box::new(Keyframes::from_pairs(keyframes))),

            // Processing
            SignalSpec::Clamp { signal, min, max } => {
                Ok(Box::new(Clamp::new(signal.build()?, *min, *max)))
            }

            SignalSpec::Quantize { signal, levels } => {
                Ok(Box::new(Quantize::new(signal.build()?, *levels)))
            }

            SignalSpec::Remap {
                signal,
                in_min,
                in_max,
                out_min,
                out_max,
            } => Ok(Box::new(Remap::new(
                signal.build()?,
                *in_min,
                *in_max,
                *out_min,
                *out_max,
            ))),

            SignalSpec::Invert { signal } => Ok(Box::new(Invert::new(signal.build()?))),

            SignalSpec::Abs { signal } => Ok(Box::new(Abs::new(signal.build()?))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_sine() {
        let spec = SignalSpec::Sine {
            frequency: 2.0,
            amplitude: 0.5,
            offset: 0.0,
            phase: 0.0,
        };
        let signal = spec.build().unwrap();
        // At t=0.125 (quarter cycle at 2Hz), should be at peak (0.5)
        assert!((signal.sample(0.125) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_build_composed() {
        let spec = SignalSpec::Add {
            a: Box::new(SignalSpec::Constant { value: 0.3 }),
            b: Box::new(SignalSpec::Constant { value: 0.4 }),
        };
        let signal = spec.build().unwrap();
        assert!((signal.sample(0.0) - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_serde_roundtrip() {
        let spec = SignalSpec::Sine {
            frequency: 2.0,
            amplitude: 1.0,
            offset: 0.5,
            phase: 0.25,
        };
        let json = serde_json::to_string(&spec).unwrap();
        let parsed: SignalSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, parsed);
    }

    #[test]
    fn test_serde_composed() {
        let spec = SignalSpec::Mix {
            a: Box::new(SignalSpec::Sine {
                frequency: 1.0,
                amplitude: 1.0,
                offset: 0.0,
                phase: 0.0,
            }),
            b: Box::new(SignalSpec::Triangle {
                frequency: 2.0,
                amplitude: 0.5,
                offset: 0.0,
                phase: 0.0,
            }),
            mix: 0.3,
        };
        let json = serde_json::to_string(&spec).unwrap();
        let parsed: SignalSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, parsed);
    }

    #[test]
    fn test_build_invalid_gaussian_returns_err() {
        let spec = SignalSpec::GaussianNoise {
            seed: 0,
            std_dev: -1.0, // Invalid: negative std_dev
            amplitude: 1.0,
            offset: 0.0,
        };
        assert!(spec.build().is_err());
    }
}

// <FILE>mixed-signals/src/types/signal_spec.rs</FILE> - <DESC>SignalSpec enum for serialization</DESC>
// <VERS>END OF VERSION: 2.0.0</VERS>
