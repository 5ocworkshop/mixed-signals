<!-- <FILE>CHEATSHEET.md</FILE> - <DESC>API reference and patterns</DESC> -->
<!-- <VERS>VERSION: 1.1.0</VERS> -->
<!-- <WCTX>Pre-launch documentation update</WCTX> -->
<!-- <CLOG>Added Physics Solvers section</CLOG> -->

# mixed-signals Cheatsheet
## Generators (Motion & Cycling)
*All outputs bipolar [-1, 1]. Use `.normalized()` for [0, 1].*
| Signal | Description | Best For... |
|--------|-------------|-------------|
| **Sine** | Smooth, organic oscillation | Breathing UI, pulsing lights |
| **Triangle** | Linear up/down ramp | Throbbers, bouncing items |
| **Square** | Hard on/off switch | Blinking cursors, strobes |
| **Sawtooth** | Ramp up, instant reset | Scrolling textures, loading bars |
| **Pulse** | High during specific window | Timed triggers, sequencers |
| **Ramp** | Linear A → B over time | Transitions, fade-ins |
| **Step** | Instant jump at threshold | State changes, hard cuts |
| **Constant** | Fixed value | Baselines, placeholders |
| **Keyframes** | Piecewise linear interpolation | Data-driven curves, custom envelopes |
```rust
// Common Pattern: Pulsing Opacity (normalized for TUI)
let s = Sine::new(1.0, 1.0, 0.0, 0.0).normalized(); // Freq, Amp, Offset, Phase

// Data-driven animation curve
let kf = Keyframes::new(vec![
    Keyframe::new(0.0, 0.0),
    Keyframe::new(0.5, 1.0),
    Keyframe::new(1.0, 0.3),
]);
```
## Noise (Texture & Variation)
*All seeded. Deterministic. Bipolar [-1, 1] output. Use `.normalized()` for [0, 1].*
| Type | Description | Best For... |
|------|-------------|-------------|
| **WhiteNoise** | Chaotic, uncorrelated | Screen shake, static, glitches |
| **PerlinNoise** | Smooth, cloud-like flow | Drifting fog, terrain, heatmaps |
| **PinkNoise** | 1/f noise (balanced) | Natural textures, audio-feel |
| **GaussianNoise** | Bell-curve distribution | Natural clustering, particle spread |
| **Correlated** | Smooth random walk | Drunk-walk motion, wandering NPCs |
| **Poisson** | Discrete event timing | Raindrops, packet bursts |
| **Spatial** | Position-based (x,y) | Procedural textures (wood, marble) |
| **PerCharacter** | Stable per-index value | Matrix rain, typewriter jitter |
| **ImpulseNoise** | Poisson-distributed events | Lightning, Geiger counters |
| **StudentTNoise** | Heavy-tailed distribution | Extreme variations, outliers |
```rust
// Common Pattern: Organic Shake (normalized for screen coordinates)
let shake = PerlinNoise::with_seed(42).with_octaves(2, 0.5).normalized();
```
### Fast Variants
For performance-critical paths (~25x faster):
- `FastSeededRandom`, `FastPinkNoise`, `FastCorrelatedNoise`
## Envelopes (Lifecycle)
*Shape how things start, sustain, and stop.*
- **Adsr**: Attack → Decay → Sustain → Release. (Classic synth/UI lifecycle)
- **LinearEnvelope**: Simple Attack → Release. (Fade-in / Fade-out)
- **Impact**: Instant Attack → Exponential Decay. (Hit markers, explosions)
## Composition (Logic)
*Combine signals to create complex behavior.*
| Operator | Description |
|----------|-------------|
| **Mix** | Linear interpolation (`lerp`) between two signals |
| **Add** | Sum signals (e.g., Signal + Noise) |
| **Multiply** | Scale/Gate (e.g., Oscillator * Envelope) |
| **Scale** | Unclamped multiplication (allows >1.0 intermediate values) |
| **Sum** | N-way addition of multiple signals |
| **FrequencyMod** | Use one signal to drive the speed of another |
| **VcaCentered** | Bipolar VCA with neutral center point (Advanced) |
## Processing (Filters)
*Core transforms:*
- **Normalized**: Map signal's output_range() to [0, 1]. Primary API for TUI.
- **Clamp**: Hard limit min/max.
- **Remap**: Map any range to any range (e.g., [-1,1] to screen coordinates).
- **Quantize**: Bit-crush / Stepped output.
- **Invert**: Negate: `-value`.
- **Abs**: `|value|`.

*Advanced (audio-grade, stateful):*
- **Biquad**: IIR filter (lowpass, highpass, bandpass, notch, allpass)
- **Svf**: State variable filter for multi-pole filtering
- **LowPass**: Simple single-pole lowpass
- **Clipper**: Soft/hard clipping (hard, soft, tanh, sine modes)

*Helpers:*
- `bipolar_to_unipolar()`, `unipolar_to_bipolar()`, `remap_range()`
## Shuffle Algorithms
*Deterministic reordering of vectors.*
| Algo | Complexity | Description |
|------|------------|-------------|
| **fisher_yates** | O(n) | Standard fair shuffle. |
| **sattolo** | O(n) | Cyclic. No item stays in original spot. |
| **partial** | O(k) | Shuffle only first k elements. |
| **weighted** | O(n log n) | Bias toward high-weight items (Loot tables). |
| **constrained** | O(n²) | Prevent consecutive repeats (Playlists). |
| **riffle** | O(n) | Simulates physical card riffle (GSR model). |
| **overhand** | O(n) | Casual card-shuffle simulation. |
| **interleave** | O(n) | Deterministic Faro shuffle. |
| **reservoir** | O(n) | Streaming/iterator inputs. |
| **smooth** | O(n²) | Minimizes jarring transitions between items. |
## Physics Solvers
*Deterministic physics for UI animations.*

| Solver | Description | Best For... |
|--------|-------------|-------------|
| **DampedSpring** | Harmonic oscillation with damping | Bouncy buttons, elastic snap-back |
| **BallisticTrajectory** | Projectile motion under gravity | Thrown objects, arc animations |
| **FrictionDecay** | Exponential velocity decay | Scroll momentum, fling gestures |
| **SimplePendulum** | Pendulum oscillation | Swinging elements, clock hands |
| **CircularOrbit** | Uniform circular motion | Rotating indicators, orbital menus |
| **PointAttractor** | Force field toward a point | Magnetic effects, gravity wells |
| **BouncingDrop** | Multi-bounce with energy loss | Drop-in modals, rubber ball physics |

```rust
// Bouncy modal drop-in
let drop = BouncingDrop::rubber_ball(0.0, 300.0, 500.0);
let y = drop.sample(t);  // Returns position with bounces

// Spring-loaded button
let spring = DampedSpring::default();
let displacement = spring.sample(t);
```

## Easing
*Make movement feel physical.*
See [easings.net](https://easings.net/) for visuals.
- **Standard:** `Quad`, `Cubic`, `Quart`, `Quint`, `Sine`, `Expo`, `Circ`
- **Effect:** `Back` (overshoot), `Elastic` (spring), `Bounce` (gravity)
- **Modes:** `In` (start slow), `Out` (end slow), `InOut` (both)
```rust
let val = ease(t, EasingType::CubicOut);
```
## Serialization (SignalSpec)
Define animations in JSON/TOML.
```json
{
  "type": "mix",
  "mix": 0.2,
  "a": { "type": "sine", "frequency": 1.0 },
  "b": { "type": "white_noise", "seed": 42 }
}
```
## Advanced: Stateful Filters
*These maintain internal state (IIR filtering). Everything else is stateless.*

| Type | Range | Notes |
|------|-------|-------|
| **Biquad** | ~Input | IIR filter (LP/HP/BP/Notch) |
| **Svf** | ~Input | State variable filter |
| **LowPass** | ~Input | One-pole smoothing |

## Getting [0, 1] Output
*All core signals output bipolar [-1, 1]. For TUI work:*

```rust
// Preferred: .normalized() auto-detects range
let opacity = Sine::default().normalized();

// Alternative: helper functions
let value = bipolar_to_unipolar(signal.sample(t));
```

<!-- <FILE>CHEATSHEET.md</FILE> - <DESC>API reference and patterns</DESC> -->
<!-- <VERS>END OF VERSION: 1.1.0</VERS> -->
