<!-- <FILE>README.md</FILE> - <DESC>mixed-signals crate overview</DESC> -->
<!-- <VERS>VERSION: 1.1.0</VERS> -->
<!-- <WCTX>Pre-launch documentation update</WCTX> -->
<!-- <CLOG>Added physics module, updated install instructions for 0.2 release</CLOG> -->

# mixed-signals

**Primitives for signals, waveforms, noise, easing, RNG, and shuffling.**

Originally built for TUI animation, `mixed-signals` provides the ergonomic "batteries included" math to make your applications feel organic without the complexity of audio DSP libraries.

## The Core Philosophy: Bipolar Core, Explicit Normalization

**This is the most important thing to know:** All core signals—oscillators, noise generators—output **bipolar [-1, 1]** by default. This matches audio/synthesis conventions and enables mathematically correct composition.

For TUI animation (opacity, brightness, progress bars), call `.normalized()` to get **[0, 1]** output:

```rust
use mixed_signals::prelude::*;

// Core signals are bipolar [-1, 1]
let sine = Sine::default();
assert_eq!(sine.sample(0.0), 0.0);   // Bipolar: center at 0
assert_eq!(sine.sample(0.25), 1.0);  // Peak at +1
assert_eq!(sine.sample(0.75), -1.0); // Trough at -1

// For TUI work, normalize to [0, 1]
let opacity = sine.normalized();
assert_eq!(opacity.sample(0.0), 0.5);  // Center maps to 0.5
assert_eq!(opacity.sample(0.25), 1.0); // Peak maps to 1.0
assert_eq!(opacity.sample(0.75), 0.0); // Trough maps to 0.0

// Remap to any range
let screen_y = Remap::new(sine, 0.0, 480.0); // -1..1 → 0..480
```

**Why bipolar?** Audio synthesis, FM modulation, and signal composition all expect bipolar signals. Clamping at every stage loses information and breaks mathematical relationships. The `.normalized()` layer gives TUI consumers a clean API without compromising the core.

## Quick Example: A "Breathing" UI Element

Make a UI component pulse smoothly, but with a slight organic jitter so it feels alive:

```rust
use mixed_signals::prelude::*;

// 1. Create a base pulse (Sine wave 0.5Hz) - outputs [-1, 1]
let pulse = Sine::new(0.5, 1.0, 0.0, 0.0);

// 2. Create subtle jitter (Perlin noise) - outputs [-1, 1]
let jitter = PerlinNoise::new(1234, 2.0, 0.5);

// 3. Compose them and normalize for TUI use
let breathing_signal = pulse.mix(jitter, 0.2).normalized();

// Sample at time t=1.5
let opacity = breathing_signal.sample(1.5);
// Returns a safe f32 between 0.0 and 1.0
```

## Install

Not yet published to crates.io (coming soon). Use a git dependency:
```toml
[dependencies]
mixed-signals = { git = "https://github.com/5ocworkshop/mixed-signals" }
```

## Why "mixed-signals"?

It's a mix of signal generators plus the utilities you inevitably need alongside them—easing, shuffles, deterministic RNG. Built for terminal animations, but the primitives have broad application. Once you have signals on tap, you find uses for them everywhere.

## Guiding Philosophy: QCIT

This library is developed under the **QCIT** principle—a virtuous cycle where each element reinforces the others:

| Pillar | Meaning |
|--------|---------|
| **Quality** | Production-grade code with tests, edge cases handled, no shortcuts |
| **Consistency** | Uniform patterns across the API—same conventions, same behavior expectations |
| **Innovation** | Solve real problems with thoughtful design, not checkbox features |
| **Trust** | The outcome earned when Q+C+I are sustained over time |

**Trust is the goal.** When you use `mixed-signals`, you should be able to rely on it without second-guessing. Every oscillator and noise generator outputs bipolar `[-1, 1]`. Every noise generator is deterministic. Every function behaves predictably. Call `.normalized()` when you need `[0, 1]`. This consistency isn't accidental—it's the result of applying QCIT at every decision point.

Trust is built incrementally through quality and consistency. One sloppy edge case or inconsistent API erodes trust faster than ten good features build it.

### Features at a Glance

- **Signals** — Time‑based functions outputting bipolar [-1, 1]; use `.normalized()` for [0, 1].
- **Generators** — 11 waveforms for driving cyclic motion (pulsing, breathing, blinking, keyframe animation).
- **Noise & Randomness** — 12 noise types for organic variation (screen shake, drift, texture), with fast variants.
- **Physics** — 7 deterministic solvers (springs, bounces, projectiles, orbits) for natural UI motion.
- **Processing** — Transform outputs: clamp, remap, quantize, invert, normalize, and more.
- **Composition** — Layer signals: add, multiply, mix, scale, sum, FM synthesis (unclamped for accuracy).
- **Easing** — 25 curves for animating motion with natural feel.
- **Context‑aware** — Phase/loop timing and per‑character randomness (Matrix rain, typewriter effects).
- **Shuffle** — 10 algorithms for reordering (loot tables, card games, playlists).
- **Advanced** — Audio-grade filters and synthesis primitives.

## Hot-Reloading with SignalSpec

Don't hardcode magic numbers. `SignalSpec` is a serde-compatible enum that builds into a live signal at runtime. This lets you define animations in config files that non-programmers can edit, or hot-reload signal parameters without recompiling:

```rust
use mixed_signals::types::SignalSpec;

// Load this from a TOML/JSON file
let spec = SignalSpec::Sine { frequency: 1.0, amplitude: 1.0, offset: 0.0, phase: 0.0 };
let signal = spec.build().unwrap();
```

## Detailed Capabilities

### Modules
- `generators` — Oscillators (Sine, Triangle, Square, Sawtooth, Pulse) and utilities (Constant, Ramp, Step, Keyframes).
- `noise` — Continuous noise (White, Perlin) for organic variation.
- `random` — 12 deterministic noise types. Same seed + time = same value. Fast variants available.
- `envelopes` — ADSR, linear, impact. Shape amplitude over time.
- `physics` — 7 deterministic solvers (DampedSpring, BouncingDrop, FrictionDecay, Pendulum, Orbit, Projectile, Attractor).
- `composition` — Combine signals (Add, Multiply, Mix, Scale, Sum, FrequencyMod).
- `processing` — Reshape outputs (Abs, Invert, Clamp, Remap, Quantize).
- `shuffle` — 10 algorithms + animators (fair shuffles, weighted draws, card-style cuts).
- `visualization` (feature) — `SignalView` widget for Ratatui.

### Noise & Randomness

All generators are seeded for reproducibility. Fast variants (using SplitMix64) are available for performance‑critical paths.

| Type | Description |
|------|-------------|
| **White** | Uniform random, uncorrelated frame‑to‑frame. |
| **Perlin** | Smooth coherent noise with configurable octaves and persistence. |
| **Gaussian** | Normal distribution, values cluster around mean. |
| **Poisson** | Models discrete event timing (network packets, glitches). |
| **Pink (1/f)** | Fractal noise, smoother than white, rougher than Perlin. |
| **Correlated** | Brownian motion / random walk with tunable correlation. |
| **Spatial** | Position‑based; same (x, y) always yields same value. |
| **PerCharacter** | Index‑based; consistent per‑character randomness. |

### Easing

25 standard easing curves organized by family, each with in/out/in‑out variants.
*Play with them here:* https://easings.net/ (not affiliated, just a great site)

| Family | Curve | Character |
|--------|-------|-----------|
| Linear | — | Constant rate |
| Quad | t² | Gentle acceleration |
| Cubic | t³ | Moderate acceleration |
| Sine | Sinusoidal | Smooth, natural feel |
| Expo | Exponential | Sharp acceleration |
| Circ | Circular | Rounded curve |
| Back | Overshoot | Anticipation/follow‑through |
| Elastic | Spring | Oscillating settle |
| Bounce | Bouncing | Multiple rebounds |

Custom curves via `solve_bezier(t, x1, y1, x2, y2)` using CSS‑compatible cubic‑bezier control points.

### Shuffle Algorithms

| Algorithm | Use Case |
|-----------|----------|
| **Fisher‑Yates** | Standard unbiased O(n) shuffle |
| **Sattolo** | Cyclic permutations, no element stays in place |
| **Partial** | Shuffle only k elements |
| **Weighted** | Priority‑biased ordering |
| **Constrained** | Enforce variety (max consecutive repeats) |
| **Riffle** | Gilbert‑Shannon‑Reeds card‑shuffle model |
| **Overhand** | Casual card‑shuffle simulation |
| **Interleave** | Deterministic Faro shuffle |
| **Reservoir** | Streaming/iterator inputs |
| **Smooth** | Minimize transition jarring |

**Animators:** `RiffleAnimator` and `OverhandAnimator` provide frame‑by‑frame shuffle visualization with state tracking.

### Physics Solvers

Deterministic physics for UI animations. All solvers use analytical solutions for framerate-independent behavior.

| Solver | Use Case |
|--------|----------|
| **DampedSpring** | Bouncy buttons, elastic snap-back, spring-loaded toggles |
| **BouncingDrop** | Modal drop-in animations, rubber ball physics |
| **FrictionDecay** | Scroll momentum, fling gestures, velocity decay |
| **SimplePendulum** | Swinging elements, pendulum clocks |
| **CircularOrbit** | Rotating indicators, orbital menu layouts |
| **BallisticTrajectory** | Thrown objects, projectile arcs |
| **PointAttractor** | Magnetic effects, gravity wells, cursor attraction |

```rust
use mixed_signals::physics::{DampedSpring, BouncingDrop};

// Bouncy modal that drops in and settles
let modal = BouncingDrop::rubber_ball(0.0, 300.0, 500.0);
let y = modal.sample(elapsed_time);

// Spring-loaded button that snaps back
let spring = DampedSpring::default();
let offset = spring.sample(elapsed_time);
```

## Design Philosophy & Guarantees

### Determinism & Context

Most signals are deterministic for the same inputs. Random/noise signals use `SignalContext` (frame/seed/phase/char_index) to keep results repeatable. This matters because animations often need to replay identically—debugging a glitchy transition is impossible if it's different every time. Seeded randomness also enables network sync: two clients with the same seed produce the same "random" particle effects without exchanging per-frame data.

```rust
let ctx = SignalContext::new(120, 7).with_char_index(3);
let noise = PerCharacterNoise::with_seed(99);
// Returns the same value every time for this context
let v = noise.sample_with_context(0.0, &ctx);
```

### Fluent API

Chain operations via `SignalExt` trait methods:
```rust
signal.add(other).scale(0.8).mix(constant, 0.3)
```

### Technical Notes

- **f64 time inputs**: `SignalTime` uses `f64` to avoid precision loss in long‑running sessions; outputs remain `f32`.
- **Phase‑aware context**: `SignalContext` supports lifecycle phases (Start, Active, End, Done, Custom) for entrance/exit effects.
- **Defensive inputs**: Non‑finite values (NaN/Inf) are sanitized to safe defaults at sample time.
- **SignalRange**: Defaults to unit range if bounds are non‑finite.

## Advanced: Filters & Stateful Processing

The library includes audio-grade filters that maintain internal state. These are the only components with special behavior—everything else is stateless.

### Stateful Filters

These filters use `Mutex`-protected internal state for IIR (infinite impulse response) filtering. They are deterministic for monotonically increasing time and thread-safe, but `sample()` mutates internal state:

| Type | Range | Notes |
|------|-------|-------|
| **Biquad** | ~Input range | IIR filter (lowpass, highpass, bandpass, notch, allpass) |
| **Svf** | ~Input range | State variable filter for multi-pole filtering |
| **LowPass** | ~Input range | Simple single-pole smoothing |

### Getting [0, 1] Output

All core signals are bipolar [-1, 1]. For TUI work:

```rust
use mixed_signals::prelude::*;

// Method 1: .normalized() - auto-detects range from signal
let opacity = Sine::default().normalized();  // [-1,1] → [0,1]

// Method 2: Clamp - hard limit without remapping
let clamped = Clamp::new(my_signal, 0.0, 1.0);

// Method 3: Helper functions
use mixed_signals::processing::{bipolar_to_unipolar, remap_range};
let value = bipolar_to_unipolar(sine.sample(t));  // -1..1 → 0..1
let screen_y = remap_range(value, SignalRange::BIPOLAR, SignalRange::new(0.0, 480.0));
```

### Additional Generators

| Generator | Description |
|-----------|-------------|
| **Keyframes** | Piecewise linear interpolation between time/value points. Data-driven animation curves. |
| **PhaseAccumulator** | Maintains continuous phase for wavetable synthesis. Enables true FM without discontinuities. Stateless (recomputes from t=0). |
| **PhaseSine** | Converts phase [0,1) to sine wave [−1,1]. Bipolar output for audio. |

### Additional Composition

| Operator | Description |
|----------|-------------|
| **Scale** | Unclamped multiplication—outputs can exceed 0..1. Use for `carrier * envelope` in audio. |
| **Sum** | N-way signal addition for combining multiple sources. |
| **VcaCentered** | Voltage-controlled amplifier with neutral center point (0.5 at zero amplitude). Outputs 0..1. |

### Audio Filters & Processing

These filters are **stateful**—they maintain internal history for IIR filtering.

| Processor | Description |
|-----------|-------------|
| **Biquad** | Second-order IIR filter with modes: lowpass, highpass, bandpass, notch, allpass. Stateful. |
| **Svf** | State variable filter for multi-pole filtering with simultaneous outputs. Stateful. |
| **LowPass** | Simple single-pole lowpass for gentle smoothing. Stateful. |
| **Clipper** | Soft/hard clipping and saturation. Modes: hard, soft, tanh, sine. Stateless. |

### Additional Noise Types

| Type | Description |
|------|-------------|
| **ImpulseNoise** | Poisson-distributed discrete events. Models packet bursts, lightning, Geiger counters. |
| **StudentTNoise** | Heavy-tailed distribution for extreme variations. More outliers than Gaussian. |

### Fast Variants

For performance-critical paths (real-time animation, games), fast variants use SplitMix64 hashing instead of ChaCha8Rng—approximately 25x faster with identical determinism guarantees:

- `FastSeededRandom` — Fast uniform random
- `FastPinkNoise` — Fast 1/f noise
- `FastCorrelatedNoise` — Fast random walk

### Bipolar Helpers

Utility functions for converting between unipolar (0..1) and bipolar (−1..1) ranges:

```rust
use mixed_signals::processing::{bipolar_to_unipolar, unipolar_to_bipolar, remap_range};

let audio_sample = -0.5;  // -1.0..1.0 range
let normalized = bipolar_to_unipolar(audio_sample);  // 0.0..1.0 range
let back = unipolar_to_bipolar(normalized);  // -1.0..1.0 range
```

## FAQ

### 25 Easing functions?! Where do I start?

Start with Quad or Cubic for subtle acceleration. Use Elastic or Bounce sparingly for playful UI. The [easings.net](https://easings.net/) interactive guide makes the differences obvious.

### I'm doing audio/engineering/precision simulation—can I use this?

This library prioritizes ergonomics for TUI animation, casual games, and exploratory dev. That said, the Advanced section includes audio-grade filters (Biquad, SVF) and synthesis primitives (PhaseAccumulator, VcaCentered) that work well for real-time audio. For production audio DSP, you may want a dedicated library, but `mixed-signals` can get you surprisingly far.

## Feature Flags

- `visualization`: enables the `SignalView` widget (ratatui). A simple demo for visualizing signals in the terminal—handy for exploring how different waveforms behave before wiring them into your application.
- `realtime-audio`: enables real-time audio playback via rodio. Requires ALSA dev headers on Linux (`libasound2-dev`). Used by the KITT scanner demo for synchronized audio.

## Docs

- `QUICKSTART.md` — 5‑minute getting started.
- `CHEATSHEET.md` — 1–2 page feature reference with call patterns.

## Examples

- `examples/snow_demo.rs` — lightweight snow demo with signal-driven drift.
- `examples/visualizer.rs` — optional signal visualizer (requires `visualization` feature).
- `examples/kitt.rs` — K.I.T.T. Larson scanner with synchronized audio (see below).

### KITT Scanner Demo

A visual Larson scanner with optional real-time synthesized audio, demonstrating keyframe envelopes, period-adaptive timing, and heartbeat-synchronized animation.

```bash
# Visual only (works everywhere)
cargo run --example kitt --features visualization

# With audio (requires audio system libraries)
cargo run --example kitt --features "visualization,realtime-audio"
```

**Controls:** `[t]` cycle modes (Alert/Cruising/Menacing), `[m]` mute, `[space]` reset, `[q]` quit.

> **Note on audio:** The `realtime-audio` feature requires ALSA development headers on Linux (`sudo apt install libasound2-dev`). This is a compile-time dependency only. At runtime, if no audio device is available (common in containers, WSL2 without audio forwarding, or headless systems), the demo gracefully shows `[NO AUDIO]` and continues with visual-only mode.

## Now go build something

If you made it this far, thank you. Now get out there and build something with mixed-signals.

## License

MIT

<!-- <FILE>README.md</FILE> - <DESC>mixed-signals crate overview</DESC> -->
<!-- <VERS>END OF VERSION: 1.1.0</VERS> -->
