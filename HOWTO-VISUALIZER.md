<!-- <FILE>HOWTO-VISUALIZER.md</FILE> - <DESC>Examples and demos guide</DESC> -->
<!-- <VERS>VERSION: 1.0.0</VERS> -->
<!-- <WCTX>Pre-launch documentation update</WCTX> -->
<!-- <CLOG>Complete rewrite covering all demos with build/run instructions</CLOG> -->

# HOWTO: Examples & Demos

This library includes interactive terminal demos showcasing signal generation, composition, and visualization. All demos use Ratatui for rendering and optionally support real-time audio synthesis.

## Prerequisites

- A terminal with ANSI color and Unicode support (for Braille rendering)
- Rust toolchain (`cargo`)
- For audio: ALSA dev headers on Linux (`sudo apt install libasound2-dev`)

## Building & Running

All demos require the `visualization` feature. Audio is optional.

```bash
# Visual only (works everywhere)
cargo run --example <demo_name> --features visualization

# With audio (requires audio libraries)
cargo run --example <demo_name> --features "visualization,realtime-audio"
```

**Press `c` in any demo to see the code snippet** showing the key signal composition used.

---

## Available Demos

### visualizer

**The signal oscilloscope.** See the mathematical structure of waveforms, noise, and envelopes in real-time before wiring them into your application.

```bash
cargo run --example visualizer --features visualization
```

**What it shows:** All signal types rendered as live waveform graphs using Braille characters for high resolution.

| Key | Action |
|-----|--------|
| **Up/Down** | Select signal |
| **Left/Right** | Adjust speed |
| **Space** | Pause/Resume |
| **Enter** | Fullscreen |
| **1-5** | Filter by category |
| **c** | Toggle gradient |
| **q** | Quit |

---

### kitt

**K.I.T.T. scanner with police lights and sirens.** Demonstrates keyframe envelopes, FM synthesis, and multiple light/sound modes.

```bash
# Visual only
cargo run --example kitt --features visualization

# With audio (synchronized sirens/sweeps)
cargo run --example kitt --features "visualization,realtime-audio"
```

**What it shows:** 7 light modes with unique synchronized audio for each.

| Mode | Visual | Audio |
|------|--------|-------|
| KITT | Larson scanner | FM sweep |
| Band/Full/Pulse | 3D cylinder rotation | Low rumble |
| US Police | Red/blue wig-wag | Wail/yelp siren |
| EU Rotate/Flash | Blue beacon | Nee-naw two-tone |

| Key | Action |
|-----|--------|
| **Space** | Cycle modes |
| **m** | Mute audio |
| **r** | Reset |
| **c** | Show code |
| **q** | Quit |

---

### decryption

**Matrix-style decryption effect.** Demonstrates `PerCharacterNoise` for stable per-index randomness.

```bash
cargo run --example decryption --features visualization

# With audio (pentatonic tones tied to cycling characters)
cargo run --example decryption --features "visualization,realtime-audio"
```

**What it shows:** Characters cycle through random symbols before revealing the final text. Each character has a unique, deterministic animation phase.

| Key | Action |
|-----|--------|
| **Space** | Restart animation |
| **c** | Show code |
| **q** | Quit |

---

### snow_demo

**Falling snow with physics modes.** Demonstrates signal-driven particle motion.

```bash
cargo run --example snow_demo --features visualization
```

**What it shows:** Snowflakes with drift, gust, swirl, and blizzard modes.

| Key | Action |
|-----|--------|
| **1-4** | Switch mode (Drift/Gust/Swirl/Blizzard) |
| **Up/Down** | Adjust flake count |
| **c** | Show code |
| **q** | Quit |

---

### skyline

**Procedural city skyline.** Demonstrates Perlin noise + Quantize for terrain generation.

```bash
cargo run --example skyline --features visualization
```

**What it shows:** Scrolling cityscape with procedurally generated building heights and randomly lit windows.

| Key | Action |
|-----|--------|
| **Left/Right** | Scroll |
| **c** | Show code |
| **q** | Quit |

---

### smart_light

**Organic "breathing" light.** Demonstrates signal composition (Sine + Noise) with the fluent API.

```bash
cargo run --example smart_light --features visualization
```

**What it shows:** A light gauge that pulses with subtle organic jitter, transitioning between idle/active/alert states.

| Key | Action |
|-----|--------|
| **Space** | Cycle states |
| **c** | Show code |
| **q** | Quit |

---

## Audio Notes

The `realtime-audio` feature requires platform audio libraries:

| Platform | Requirement |
|----------|-------------|
| Linux | `libasound2-dev` (ALSA) |
| macOS | Built-in CoreAudio |
| Windows | Built-in WASAPI |

If no audio device is available (WSL2 without audio, containers, headless systems), demos gracefully fall back to visual-only mode showing `[NO AUDIO]`.

**Audio demonstrates waveform synthesis in practice.** The same signals driving visual animations can generate sound—the KITT demo's FM sweep uses the exact same `Sine` and `FrequencyMod` primitives you'd use for UI animation.

---

## Implementation Notes

- Demos use `ratatui` with Braille rendering (2x4 patterns: `⣿`) for smooth curves
- All signal sampling is deterministic—same time = same output
- Press `c` to see the core signal composition code used in each demo

<!-- <FILE>HOWTO-VISUALIZER.md</FILE> - <DESC>Examples and demos guide</DESC> -->
<!-- <VERS>END OF VERSION: 1.0.0</VERS> -->
