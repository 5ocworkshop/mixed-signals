<!-- <FILE>CHANGELOG.md</FILE> - <DESC>mixed-signals release notes</DESC> -->
<!-- <VERS>VERSION: 0.2.0</VERS> -->
<!-- <WCTX>Release documentation for crates.io</WCTX> -->
<!-- <CLOG>Updated for 0.2.0 release with physics module and bipolar core</CLOG> -->

# Changelog
All notable changes to this project will be documented in this file.

## [0.2.0] - 2026-01-02

### Added
- **Physics module** with 7 deterministic solvers: `DampedSpring`, `BallisticTrajectory`, `FrictionDecay`, `SimplePendulum`, `CircularOrbit`, `PointAttractor`, `BouncingDrop`.
- **Property-based testing** for signal invariants (output bounds, determinism).
- **Fast random** module with CPU feature detection and AVX2 batch operations (~25x faster).
- **New examples**: Police sirens, cylinder rotation sounds, and decryption audio demo.
- `ImpulseNoise` for Poisson-distributed discrete events.
- `StudentTNoise` for heavy-tailed distributions with more outliers than Gaussian.

### Changed
- **BREAKING: Bipolar core architecture** — All oscillators and noise generators now output **[-1, 1]** by default. Use `.normalized()` to get [0, 1] for TUI work.
- Composition operators (Add, Multiply, Mix) no longer clamp, enabling proper audio/synthesis workflows.
- Processing operators (Abs, Invert, Quantize) no longer clamp.
- `.invert()` now negates the value (bipolar-correct) rather than `1.0 - value`.
- Refactored noise helpers into core module for deduplication.

### Fixed
- Clippy warnings addressed across codebase.
- Physics solver robustness, precision, and performance improvements.

### Migration
For TUI animations expecting [0, 1] output:
```rust
// Before (0.1.x)
let opacity = Sine::default().sample(t);

// After (0.2.x)
let opacity = Sine::default().normalized().sample(t);
```

## [0.1.0] - 2025-12-26
### Added
- Core `Signal` trait and context system.
- Oscillator generators (sine, triangle, square, sawtooth) and utility signals (ramp, step, pulse, constant).
- Noise/random signals: white, Perlin‑like, Gaussian, Poisson, pink, correlated, spatial, per‑character, seeded random.
- Envelopes: ADSR, linear, impact.
- Composition and processing operators.
- Deterministic shuffling algorithms + animators.
- `SignalSpec` / `SignalOrFloat` for serializable specs.
- Visualization feature (`SignalView`).

### Changed
- Invalid inputs (NaN/Inf) are sanitized to defaults at sample time to keep outputs finite.

<!-- <FILE>CHANGELOG.md</FILE> - <DESC>mixed-signals release notes</DESC> -->
<!-- <VERS>END OF VERSION: 0.2.0</VERS> -->
