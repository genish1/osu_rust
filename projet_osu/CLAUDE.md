# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Run the game
cargo run

# Check compilation without producing a binary
cargo check

# Run tests
cargo test

# Run a specific test
cargo test <test_name>
```

> Note: `.cargo/config.toml` forces software rendering (`LIBGL_ALWAYS_SOFTWARE=1`) and disables Wayland (`WAYLAND_DISPLAY=""`). This is required for WSL2 compatibility.

## Architecture

This is an osu!-like rhythm game built with **Bevy 0.18**. The game loads a custom `.osumap` beatmap, displays hit circles synchronized to music, and scores player input based on timing precision.

### Module overview

| Module | Role |
|---|---|
| `main.rs` | Entry point — creates the Bevy `App`, inserts resources, registers systems |
| `beatmap.rs` | Parses `.osumap` files into a `Beatmap` struct |
| `hitobject.rs` | Data types: `HitObject` enum (`Circle` / `Slider`), both carry position and `time_ms` |
| `game.rs` | `GameState` resource: score, combo, timing evaluation (`Hit300/100/50/Miss`) |
| `renderer.rs` | Bevy systems: spawns `HitCircle` + `ApproachRing` entities at startup, animates rings each frame, despawns expired circles |
| `audio.rs` | `MusicTimer` resource (elapsed seconds); loads audio from beatmap metadata and ticks the timer each frame |
| `input.rs` | Handles mouse left-click and X/C keys; maps cursor to world coordinates, finds the nearest circle within 30px, evaluates timing, updates `GameState` |

### Beatmap format (`.osumap`)

```
title = Level name
audio = filename.ogg       # relative to assets/audio/
bpm = 120

# Hit objects: type | x | y | time_ms
circle | 400 | 300 | 1500
slider | 400 | 300 | 5000 | 600 | 300 | 6000   # x_end | y_end | end_time_ms
```

Beatmap files live in `assets/maps/`. Audio files live in `assets/audio/`. Maps are embedded at compile time via `include_str!` with absolute paths in `main.rs`.

### Coordinate system

Beatmap coordinates use top-left origin (osu! convention). The renderer converts to Bevy world coordinates (center origin) as:
```
world_x = beatmap_x - window_width / 2
world_y = window_height / 2 - beatmap_y
```

The same conversion is applied in `input.rs` when mapping cursor position to world space.

### Approach ring animation

Rings appear 800ms before a circle's `time_ms`. They shrink from scale 3.0 → 1.0 over that window. When `scale <= 1.0` the ring entity is despawned.

### Timing windows

| Result | Delta |
|---|---|
| Hit300 | ≤ 200ms |
| Hit100 | 201–400ms |
| Hit50 | 401–450ms |
| Miss | > 450ms |

Score = `result.points() * current_combo`. Combo resets on Miss.
