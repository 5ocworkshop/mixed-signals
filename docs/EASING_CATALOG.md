<!-- <FILE>docs/EASING_CATALOG.md</FILE> - <DESC>Complete catalog of easing curves</DESC> -->
<!-- <VERS>VERSION: 1.0.0</VERS> -->
<!-- <WCTX>Documentation for easing module</WCTX> -->
<!-- <CLOG>Initial easing catalog for mixed-signals</CLOG> -->

# Easing Curves Catalog

Complete reference for all easing curves available in `mixed-signals`.

**Total curves**: 25 built-in types

---

## Table of Contents

- [Linear & Polynomial](#linear--polynomial)
- [Trigonometric](#trigonometric)
- [Back (Overshoot)](#back-overshoot)
- [Elastic (Spring)](#elastic-spring)
- [Bounce](#bounce)
- [Exponential](#exponential)
- [Circular](#circular)
- [Usage Examples](#usage-examples)
- [Choosing the Right Curve](#choosing-the-right-curve)

---

## Linear & Polynomial

### Linear
**Constant velocity, no acceleration**

```rust
use mixed_signals::easing::{ease, EasingType};

let y = ease(0.5, EasingType::Linear);  // Returns 0.5
```

**Behavior**: `y = x` (identity function)
**Use cases**: Constant speed motion, mechanical movements, scrolling
**Performance**: Fastest (~1ns)

---

### QuadIn, QuadOut, QuadInOut
**Gentle acceleration/deceleration**

```rust
EasingType::QuadIn     // Accelerate
EasingType::QuadOut    // Decelerate
EasingType::QuadInOut  // Smooth both ends
```

**Behavior**: Quadratic curve (`y = x²` or variants)
**Use cases**: Subtle emphasis, UI element transitions, gentle fades
**Performance**: Very fast (~2ns)

---

### CubicIn, CubicOut, CubicInOut
**Moderate acceleration/deceleration**

```rust
EasingType::CubicIn     // Accelerate
EasingType::CubicOut    // Decelerate
EasingType::CubicInOut  // Smooth both ends
```

**Behavior**: Cubic curve (`y = x³` or variants)
**Use cases**: Standard UI animations, modal entrances, panel slides
**Performance**: Very fast (~3ns)

---

## Trigonometric

### SineIn, SineOut, SineInOut
**Smooth, organic acceleration**

```rust
EasingType::SineIn     // Accelerate
EasingType::SineOut    // Decelerate
EasingType::SineInOut  // Smooth both ends
```

**Behavior**: Sine wave quarter-cycle
**Use cases**: Natural feeling motion, breathing effects, gentle emphasis
**Performance**: Fast (~8ns with trig functions)

---

## Back (Overshoot)

### BackIn, BackOut, BackInOut
**Anticipation/overshoot effects**

```rust
EasingType::BackIn     // Pull back before starting
EasingType::BackOut    // Overshoot then settle
EasingType::BackInOut  // Both effects
```

**Behavior**: Cubic with overshoot (goes <0 or >1)
**Use cases**: Playful UI, attention-grabbing, cartoonish effects
**Performance**: Fast (~5ns)
**Note**: Values exceed [0,1] range - design UI accordingly

---

## Elastic (Spring)

### ElasticIn, ElasticOut, ElasticInOut
**Oscillating spring effects**

```rust
EasingType::ElasticIn     // Spring compression
EasingType::ElasticOut    // Spring release
EasingType::ElasticInOut  // Both effects
```

**Behavior**: Exponentially damped sine wave
**Use cases**: Bouncy buttons, spring animations, playful feedback
**Performance**: Moderate (~12ns with pow + sin)
**Note**: Significant overshoot - use sparingly

---

## Bounce

### BounceIn, BounceOut, BounceInOut
**Ball bouncing effects**

```rust
EasingType::BounceIn     // Accelerate with bounces
EasingType::BounceOut    // Decelerate with bounces
EasingType::BounceInOut  // Both effects
```

**Behavior**: Multiple parabolic arcs simulating gravity
**Use cases**: Playful transitions, attention effects, game-like UI
**Performance**: Fast (~6ns)
**Note**: Multiple direction changes - not suitable for all contexts

---

## Exponential

### ExpoIn, ExpoOut, ExpoInOut
**Dramatic acceleration - stay low, then snap**

```rust
EasingType::ExpoIn     // Very slow start, explosive finish
EasingType::ExpoOut    // Explosive start, slow finish
EasingType::ExpoInOut  // Explosive both ends
```

**Behavior**: `2^(10(t-1))` and variants
**Use cases**:
- **ExpoIn**: Countdown timers, dramatic reveals, tension building
- **ExpoOut**: Quick dismiss, snappy responses, instant feedback
- **ExpoInOut**: Attention-grabbing transitions, modal dialogs

**Performance**: Fast (~10ns with `powf`)
**Visual signature**: Stays very flat early, then rockets to finish

---

## Circular

### CircIn, CircOut, CircInOut
**Smooth, gentle curves - smoother than Cubic**

```rust
EasingType::CircIn     // Gentle acceleration
EasingType::CircOut    // Gentle deceleration
EasingType::CircInOut  // Smooth both ends
```

**Behavior**: Quarter-circle arc (`sqrt(1-x²)` variants)
**Use cases**:
- **CircIn**: Smooth zoom-ins, focus transitions
- **CircOut**: Gentle settle effects, smooth stops
- **CircInOut**: Polished modal transitions

**Performance**: Fast (~8ns with `sqrt`)
**Visual signature**: Smoother than Quad/Cubic, less aggressive than Expo

---

## Usage Examples

### Basic Usage

```rust
use mixed_signals::easing::{ease, EasingType};

let t = 0.5;  // Progress (0.0 to 1.0)
let eased = ease(t, EasingType::CubicOut);
```

### Animating a Value

```rust
use mixed_signals::easing::{ease, EasingType};

fn animate(start: f32, end: f32, progress: f64, easing: EasingType) -> f32 {
    let t = ease(progress, easing);
    start + (end - start) * t
}

// Animate from 0 to 100 with bounce
let value = animate(0.0, 100.0, 0.7, EasingType::BounceOut);
```

### Serde Integration

```rust
use mixed_signals::easing::EasingType;

// Deserialize from JSON (supports both snake_case and PascalCase)
let easing: EasingType = serde_json::from_str("\"expo_out\"").unwrap();
let easing: EasingType = serde_json::from_str("\"ExpoOut\"").unwrap();  // Also works
```

---

## Choosing the Right Curve

### Decision Tree

1. **Need instant/snappy feel?** → ExpoOut
2. **Building tension?** → ExpoIn
3. **Professional, polished?** → CircOut or CubicOut
4. **Playful/attention-grabbing?** → Bounce, Elastic, or Back
5. **Natural, organic?** → SineOut
6. **Constant speed?** → Linear
7. **Not sure?** → Start with `CubicOut` (safest default)

### Common Patterns

| Use Case | Recommended | Why |
|----------|-------------|-----|
| Modal dialog enter | ExpoOut | Snappy appearance |
| Modal dialog exit | CircIn | Smooth disappearance |
| Button press | QuadOut | Subtle feedback |
| Notification slide-in | CubicOut | Professional |
| Error shake | Elastic | Attention-grabbing |
| Loading spinner | Linear | Constant rotation |
| Drawer slide | CircOut | Smooth mechanical feel |
| Dramatic reveal | ExpoIn | Builds anticipation |

---

## Performance Comparison

| Curve Type | Approximate Time | Notes |
|------------|------------------|-------|
| Linear | ~1ns | Trivial |
| Quad/Cubic | ~2-3ns | Simple math |
| Back/Bounce | ~5-6ns | Conditionals |
| Sine/Circ | ~8ns | Single trig/sqrt |
| Expo | ~10ns | Single powf |
| Elastic | ~12ns | powf + sin |

All curves are **zero-allocation** and suitable for per-frame animation.

---

## References

- **Physics solvers**: For spring/bounce physics, see `PHYSICS_SOLVERS.md`
- **Robert Penner Easing Functions**: Standard animation curves
- **easings.net**: Visual easing curve reference

---

<!-- <FILE>docs/EASING_CATALOG.md</FILE> - <DESC>Complete catalog of easing curves</DESC> -->
<!-- <VERS>END OF VERSION: 1.0.0</VERS> -->
