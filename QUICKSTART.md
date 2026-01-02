<!-- <FILE>QUICKSTART.md</FILE> - <DESC>5-minute getting started guide</DESC> -->
<!-- <VERS>VERSION: 0.3.0</VERS> -->
<!-- <WCTX>Pre-launch documentation update</WCTX> -->
<!-- <CLOG>Updated for bipolar core architecture</CLOG> -->

# Quickstart
Go from zero to animated in about 5 minutes.

## 1. Installation
Add the library to your `Cargo.toml`.
*(Not yet on crates.io—use git dependency for now)*
```toml
[dependencies]
mixed-signals = { git = "https://github.com/5ocworkshop/mixed-signals" }
```

## 2. The "Hello World" Signal
Import the prelude, create a signal, and sample it.

**Core principle:** All signals output **bipolar [-1, 1]**. Use `.normalized()` for TUI work.
```rust
use mixed_signals::prelude::*;

fn main() {
    // A standard 1Hz sine wave (bipolar: -1.0 to 1.0)
    let sine = Sine::default();

    // Sample at t = 0.25 seconds (peak of sine)
    let value = sine.sample(0.25);
    println!("Bipolar value: {}", value); // Output: 1.0

    // For TUI (opacity, progress bars): normalize to 0.0-1.0
    let opacity = sine.normalized();
    println!("Normalized: {}", opacity.sample(0.25)); // Output: 1.0
    println!("Normalized: {}", opacity.sample(0.0));  // Output: 0.5 (center)
}
```

## 3. Adding Organic Noise
Static waves are boring. Add "life" using noise.

**Note:** Noise requires a `SignalContext` to be deterministic (reproducible).
```rust
// 1. Create generators (both output bipolar [-1, 1])
let sine = Sine::new(1.0, 1.0, 0.0, 0.0); // The pulse
let noise = PerlinNoise::with_seed(42);   // The organic jitter

// 2. Mix them and normalize for TUI
// Blend: 80% Sine, 20% Noise, then map to [0, 1]
let organic_pulse = sine.mix(noise, 0.2).normalized();

// 3. Create Context (Frame 100, Seed 42)
// Ensures frame 100 always looks the same, even on different machines.
let ctx = SignalContext::new(100, 42);

// 4. Sample
let val = organic_pulse.sample_with_context(0.5, &ctx);
// Returns a safe f32 between 0.0 and 1.0
```

## 4. Per-Character Animation (The "Matrix" Effect)
If you are building a TUI, you often want characters to animate independently but coherently.
```rust
let text = "LOADING...";
let noise = PerCharacterNoise::with_seed(99).normalized();

for (index, char) in text.chars().enumerate() {
    // Context carries the character index!
    let ctx = SignalContext::new(100, 42).with_char_index(index);

    // Each character gets a unique, stable random value [0, 1]
    let alpha = noise.sample_with_context(0.5, &ctx);
    println!("Char '{}' alpha: {:.2}", char, alpha);
}
```

## 5. Hot-Reloading (Data Driven)
Don't recompile to change animation speeds. Define signals in JSON/TOML and load them at runtime.
```rust
use mixed_signals::types::SignalSpec;

// Imagine this came from "config.json"
let json = r#"{
    "type": "sine",
    "frequency": 2.0,
    "amplitude": 1.0,
    "phase": 0.5
}"#;

let spec: SignalSpec = serde_json::from_str(json).unwrap();
let signal = spec.build().unwrap();
let v = signal.sample(0.1);
```

## 6. Physics-Based Animation
Use deterministic physics solvers for natural motion.
```rust
use mixed_signals::physics::{DampedSpring, BouncingDrop};

// Bouncy modal drop-in (starts at y=0, drops 300px, settles at 500ms)
let drop = BouncingDrop::rubber_ball(0.0, 300.0, 500.0);
let y_position = drop.sample(t);

// Spring-loaded button snap-back
let spring = DampedSpring::default();
let displacement = spring.sample(t);
```

## Next Steps
- **Explore:** Run the demos: `cargo run --example visualizer --features visualization`
- **Reference:** Check `CHEATSHEET.md` for the full API reference.
- **Demos:** See `HOWTO-VISUALIZER.md` for all interactive examples.

<!-- <FILE>QUICKSTART.md</FILE> - <DESC>5-minute getting started guide</DESC> -->
<!-- <VERS>END OF VERSION: 0.3.0</VERS> -->
