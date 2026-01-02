<!-- <FILE>docs/PHYSICS_SOLVERS.md</FILE> - <DESC>Physics solvers catalog for UI animations</DESC> -->
<!-- <VERS>VERSION: 1.0.0</VERS> -->
<!-- <WCTX>Documentation for physics module</WCTX> -->
<!-- <CLOG>Initial physics solvers catalog</CLOG> -->

# Physics Solvers Catalog

Deterministic physics solvers for UI animations and simulations. All solvers implement the `Signal` trait for seamless composition with existing signal infrastructure.

**Total solvers**: 7 physics primitives

---

## Table of Contents

- [DampedSpring](#dampedspring)
- [BouncingDrop](#bouncingdrop)
- [FrictionDecay](#frictiondecay)
- [BallisticTrajectory](#ballistictrajectory)
- [SimplePendulum](#simplependulum)
- [CircularOrbit](#circularorbit)
- [PointAttractor](#pointattractor)
- [Choosing the Right Solver](#choosing-the-right-solver)
- [Usage Examples](#usage-examples)

---

## DampedSpring

**Harmonic motion with configurable damping**

```rust
use mixed_signals::physics::DampedSpring;
use mixed_signals::traits::Signal;

let spring = DampedSpring::default();
let displacement = spring.sample(0.5);
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `mass` | f32 | Mass of object (kg). Default: 1.0 |
| `stiffness` | f32 | Spring constant (k). Higher = stiffer |
| `damping` | f32 | Damping coefficient (c). Higher = more friction |
| `v0` | f32 | Initial velocity |
| `x0` | f32 | Initial displacement from equilibrium |

### Damping Regimes

| Regime | Behavior | Use Case |
|--------|----------|----------|
| **Underdamped** | Oscillates before settling | Bouncy buttons, spring animations |
| **Critically damped** | Fastest approach, no overshoot | Smooth modal transitions |
| **Overdamped** | Slow, sluggish approach | Deliberate, heavy elements |

### Convenience Constructors

```rust
// Slightly underdamped (default) - natural for UI
let bouncy = DampedSpring::default();

// Critically damped - smoothest animation
let smooth = DampedSpring::critically_damped(100.0, 1.0);

// Custom stiffness, no damping
let stiff = DampedSpring::with_stiffness(200.0);
```

### UI Applications

- **Bouncy buttons**: Underdamped spring for playful feedback
- **Modal dialogs**: Critically damped for professional feel
- **Panel slides**: Adjust stiffness for snap speed
- **Rubber-band scrolling**: Overdamped for resistance effect

---

## BouncingDrop

**Multi-bounce with energy loss per impact**

```rust
use mixed_signals::physics::BouncingDrop;
use mixed_signals::traits::Signal;

let drop = BouncingDrop::rubber_ball(0.0, 300.0, 500.0);
let height = drop.sample(0.5);
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `start_height` | f32 | Y position at t=0 |
| `ground_height` | f32 | Floor level |
| `gravity` | f32 | Acceleration |
| `restitution` | f32 | Energy retained per bounce (0.0-1.0) |

### Restitution Values

| Value | Behavior | Material Analogy |
|-------|----------|------------------|
| 1.0 | Bounces forever | Super ball |
| 0.6 | Typical bounce | Rubber ball |
| 0.3 | Low bounce | Tennis ball |
| 0.0 | No bounce | Clay |

### Convenience Constructors

```rust
// Rubber ball (restitution = 0.6)
let bouncy = BouncingDrop::rubber_ball(0.0, 300.0, 500.0);

// No bounce - stops on impact
let drop = BouncingDrop::no_bounce(0.0, 300.0, 500.0);
```

### Helper Methods

```rust
let drop = BouncingDrop::default();

// Time when first hitting ground
let t_impact = drop.time_to_first_bounce();

// Total duration until motion stops
let duration = drop.duration_until_stop();
```

### UI Applications

- **Modal drop-in**: Element drops from above and settles
- **Notification entrance**: Bouncy arrival effect
- **Error shake**: Combined with horizontal offset
- **Game-like UI**: Playful, physics-based interactions

---

## FrictionDecay

**Exponential velocity decay for scrolling and flinging**

```rust
use mixed_signals::physics::FrictionDecay;
use mixed_signals::traits::Signal;

let scroll = FrictionDecay::new(500.0, 3.0);
let offset = scroll.sample(0.5);
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `v0` | f32 | Initial velocity (units/sec) |
| `drag` | f32 | Decay rate. Higher = faster stop |

### Physics Model

```
velocity(t) = v0 * e^(-drag * t)
offset(t) = (v0 / drag) * (1 - e^(-drag * t))
max_offset = v0 / drag  (as t → ∞)
```

### Convenience Constructors

```rust
// Light friction - long, smooth scroll
let smooth = FrictionDecay::light(500.0);  // drag = 2.0

// Heavy friction - quick stop
let snappy = FrictionDecay::heavy(500.0);  // drag = 8.0
```

### Helper Methods

```rust
let decay = FrictionDecay::new(200.0, 4.0);

// Current velocity at time t
let v = decay.velocity_at(0.5);

// Time until velocity drops below threshold
let t_stop = decay.duration_until_stop(1.0);

// Maximum displacement
let max = decay.max_offset();  // 200/4 = 50
```

### UI Applications

- **Scroll momentum**: Fling gesture with natural deceleration
- **Swipe-to-dismiss**: Velocity-based dismissal
- **Carousel flick**: Momentum-based navigation
- **Sliding panels**: Inertial slide with friction

---

## BallisticTrajectory

**Projectile motion under gravity**

```rust
use mixed_signals::physics::BallisticTrajectory;

let toss = BallisticTrajectory::toss(0.0, 100.0, 50.0, -200.0, 500.0);
let (x, y) = toss.position_at(0.5);
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `start_x` | f32 | Starting X position |
| `start_y` | f32 | Starting Y position |
| `v0_x` | f32 | Initial X velocity |
| `v0_y` | f32 | Initial Y velocity (negative = up in UI coords) |
| `gravity` | f32 | Acceleration (positive = down) |
| `ground_y` | Option<f32> | Optional ground collision plane |

### Convenience Constructors

```rust
// Simple drop from height
let drop = BallisticTrajectory::drop_from(0.0, 300.0, 500.0);

// Toss with velocity (no ground)
let toss = BallisticTrajectory::toss(0.0, 100.0, 50.0, -100.0, 500.0);
```

### Methods

```rust
let traj = BallisticTrajectory::default();

// Position at time t
let (x, y) = traj.position_at(0.5);

// Velocity at time t
let (vx, vy) = traj.velocity_at(0.5);

// Time to reach ground (if ground_y set)
if let Some(t) = traj.time_to_ground() {
    println!("Impacts at t={}", t);
}
```

### UI Applications

- **Toss-to-dismiss**: Flick element off screen
- **Gravity drop**: Elements falling into place
- **Arc transitions**: Parabolic path between points
- **Particle effects**: Sparks, confetti, debris

---

## SimplePendulum

**Pendulum oscillation with damping**

```rust
use mixed_signals::physics::SimplePendulum;
use mixed_signals::traits::Signal;

let pendulum = SimplePendulum::damped(1.0, 0.2, 0.1);
let angle = pendulum.sample(0.5);  // radians
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `length` | f32 | Pendulum length (meters) |
| `gravity` | f32 | Gravitational acceleration. Default: 9.8 |
| `theta0` | f32 | Initial angle (radians). Keep small (<15°) |
| `damping` | f32 | Damping coefficient. 0 = no damping |

### Convenience Constructors

```rust
// Earth gravity, undamped
let swing = SimplePendulum::earth(1.0, 0.2);

// With damping
let damped = SimplePendulum::damped(1.0, 0.2, 0.1);
```

### Helper Methods

```rust
let pendulum = SimplePendulum::default();

// Natural frequency ω = √(g/L)
let omega = pendulum.natural_frequency();

// Period T = 2π/ω
let period = pendulum.period();

// Angular velocity at time t
let omega_t = pendulum.angular_velocity_at(0.5);
```

### UI Applications

- **Hanging signs**: Swinging notification badges
- **Hinge animations**: Door/panel opening effects
- **Metronome UI**: Rhythmic visual feedback
- **Dangling elements**: Charms, tags, decorations

---

## CircularOrbit

**Uniform circular motion**

```rust
use mixed_signals::physics::CircularOrbit;
use mixed_signals::traits::Signal;

let orbit = CircularOrbit::one_hz(100.0, 100.0, 50.0);
let (x, y) = orbit.position_at(0.25);  // Quarter revolution
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `center_x` | f32 | Center X coordinate |
| `center_y` | f32 | Center Y coordinate |
| `radius` | f32 | Orbit radius |
| `angular_velocity` | f32 | Speed (rad/sec). Positive = counter-clockwise |
| `start_phase` | f32 | Starting angle (radians) |

### Convenience Constructors

```rust
// Centered at origin
let orbit = CircularOrbit::centered(50.0, 2.0);

// One revolution per second
let spinner = CircularOrbit::one_hz(100.0, 100.0, 50.0);
```

### Methods

```rust
let orbit = CircularOrbit::default();

// Position at time t
let (x, y) = orbit.position_at(0.5);

// Velocity at time t
let (vx, vy) = orbit.velocity_at(0.5);

// Wrapped angle [0, 2π)
let angle = orbit.angle_at(0.5);

// Unwrapped angle (can exceed 2π)
let total_rotation = orbit.angle_unwrapped(10.0);

// Period of one revolution
let period = orbit.period();
```

### UI Applications

- **Loading spinners**: Rotating indicator
- **Radial menus**: Items orbiting a center
- **Satellite elements**: Decorative rotating items
- **Clock hands**: Continuous rotation

---

## PointAttractor

**Inverse-square force field toward a point**

```rust
use mixed_signals::physics::PointAttractor;

let attractor = PointAttractor::new(100.0, 100.0, 500.0);
let (fx, fy) = attractor.force_at(150.0, 100.0);
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `target_x` | f32 | Attractor center X |
| `target_y` | f32 | Attractor center Y |
| `strength` | f32 | Force magnitude. Positive = attract, negative = repel |

### Convenience Constructors

```rust
// Attractor at origin
let gravity_well = PointAttractor::at_origin(500.0);

// Repulsive field (negative strength)
let repulsor = PointAttractor::at_origin(-500.0);
```

### Methods

```rust
let attractor = PointAttractor::default();

// Force vector on particle at (px, py)
let (fx, fy) = attractor.force_at(50.0, 50.0);

// Force magnitude at distance
let force = attractor.force_magnitude_at(10.0);

// Distance from center to point
let dist = attractor.distance_to(50.0, 50.0);
```

### Signal Integration

When used as a `Signal`, `sample(t)` treats `t` as distance and returns force magnitude:

```rust
let attractor = PointAttractor::at_origin(100.0);
let force_at_distance_5 = attractor.sample(5.0);
```

### UI Applications

- **Snap-to-point**: Magnetic alignment
- **Bubble effects**: Elements attracted to cursor
- **Gravity wells**: Drag elements toward target
- **Repulsion zones**: Keep elements apart

---

## Choosing the Right Solver

### Decision Tree

```
What motion behavior do you need?

├─ Oscillating back and forth?
│  ├─ Around equilibrium point → DampedSpring
│  └─ Swinging from pivot → SimplePendulum
│
├─ Falling with bounces?
│  └─ BouncingDrop
│
├─ Decelerating to stop?
│  ├─ With bounces → BouncingDrop (low restitution)
│  └─ Smooth decay → FrictionDecay
│
├─ Projectile arc?
│  └─ BallisticTrajectory
│
├─ Continuous rotation?
│  └─ CircularOrbit
│
└─ Force toward/away from point?
   └─ PointAttractor
```

### Common UI Patterns

| UI Pattern | Recommended Solver | Configuration |
|------------|-------------------|---------------|
| Modal entrance | DampedSpring | Critically damped |
| Bouncy button | DampedSpring | Underdamped, low damping |
| Scroll momentum | FrictionDecay | Light drag |
| Notification drop | BouncingDrop | Rubber ball preset |
| Loading spinner | CircularOrbit | one_hz constructor |
| Swipe dismiss | FrictionDecay or BallisticTrajectory | Based on velocity |
| Hanging badge | SimplePendulum | Small angle, light damping |
| Magnetic snap | PointAttractor | Positive strength |

---

## Usage Examples

### Modal Drop-In Animation

```rust
use mixed_signals::physics::BouncingDrop;
use mixed_signals::traits::Signal;

// Modal drops from off-screen, bounces into place
let drop = BouncingDrop::new(
    -100.0,  // Start above viewport
    200.0,   // Final position
    800.0,   // Fast gravity
    0.4,     // Moderate bounce
);

fn animate_modal(t: f64) -> f32 {
    drop.sample(t)
}
```

### Spring-Based Button Press

```rust
use mixed_signals::physics::DampedSpring;
use mixed_signals::traits::Signal;

// Button scales down then springs back
let spring = DampedSpring::new(
    1.0,    // mass
    300.0,  // high stiffness (snappy)
    15.0,   // moderate damping
    0.0,    // no initial velocity
    0.2,    // 20% displacement (scale to 0.8)
);

fn button_scale(t: f64) -> f32 {
    1.0 - spring.sample(t)  // Invert: 1.0 → 0.8 → 1.0
}
```

### Scroll Momentum

```rust
use mixed_signals::physics::FrictionDecay;
use mixed_signals::traits::Signal;

// User flicks with velocity, content decelerates
fn create_scroll(initial_velocity: f32) -> FrictionDecay {
    FrictionDecay::new(initial_velocity, 3.0)
}

let scroll = create_scroll(500.0);
let offset_at_half_second = scroll.sample(0.5);
let final_offset = scroll.max_offset();
```

### Orbiting Menu Items

```rust
use mixed_signals::physics::CircularOrbit;
use std::f32::consts::TAU;

// 6 items evenly spaced around center
fn create_menu_orbits(center_x: f32, center_y: f32, radius: f32) -> Vec<CircularOrbit> {
    (0..6).map(|i| {
        let phase = (i as f32 / 6.0) * TAU;
        CircularOrbit::new(center_x, center_y, radius, 0.5, phase)
    }).collect()
}
```

---

## Performance Notes

All physics solvers use **analytical solutions** where possible, providing:

- **Framerate independence**: Same result regardless of sample rate
- **Zero allocation**: No heap allocations during sampling
- **Deterministic**: Same inputs always produce same outputs
- **Fast**: Typical sample time < 100ns

The solvers handle edge cases gracefully:
- NaN inputs are sanitized to safe defaults
- Zero/negative parameters have sensible fallbacks
- Large time values use phase wrapping for numerical stability

---

## References

- **Signal trait**: All solvers implement `mixed_signals::traits::Signal`
- **Easing curves**: For non-physics animations, see `EASING_CATALOG.md`

---

<!-- <FILE>docs/PHYSICS_SOLVERS.md</FILE> - <DESC>Physics solvers catalog for UI animations</DESC> -->
<!-- <VERS>END OF VERSION: 1.0.0</VERS> -->
