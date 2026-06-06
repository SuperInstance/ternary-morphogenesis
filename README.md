# ternary-morphogenesis

Alan Turing's reaction-diffusion on ternary grids — how patterns emerge from three-state chemicals diffusing at different rates.

## Why This Exists

In 1952, Turing published "The Chemical Basis of Morphogenesis," showing how two chemicals diffusing at different rates can spontaneously form patterns — spots, stripes, labyrinths. This explains how leopards get their spots, how fingers form on a hand, how zebrafish get their stripes.

Classical reaction-diffusion uses continuous concentration fields. But many real systems are effectively ternary: a cell is inactive, differentiating toward one fate, or differentiating toward another. A pixel in a pattern is background, foreground, or boundary. Continuous concentrations are often an abstraction over what is fundamentally a discrete decision.

This crate implements Turing's reaction-diffusion on ternary grids `{-1, 0, +1}`. Two ternary chemicals — an activator and an inhibitor — diffuse at different rates and interact through a Gray-Scott-style reaction. The result: emergent patterns from purely local ternary interactions, with no global coordination.

`#![no_std]` compatible — runs on embedded systems and in WASM.

## Architecture

```
TernaryGrid (width × height, cells: Vec<i8>)
    ├── get/set/clamp (values always in {-1, 0, +1})
    ├── neighbor_counts(x, y) → (neg, zero, pos)
    ├── majority(x, y) → most common neighbor value
    └── laplacian(x, y) → ∑neighbors - count×center, clamped to ternary

ReactionDiffusion
    ├── activator: TernaryGrid    (short-range chemical)
    ├── inhibitor: TernaryGrid    (long-range chemical)
    ├── da: i8, di: i8            (diffusion rates, ternary)
    ├── feed: i8, kill: i8        (reaction parameters, ternary)
    │
    ├── seed_spots(centers)       → place activator at specific points
    ├── seed_stripes(spacing)     → initialize stripe pattern
    ├── step()                    → one reaction-diffusion iteration
    ├── run(n)                    → n iterations
    └── pattern_diversity()       → count distinct 3×3 patches

is_turing_unstable(rd) → bool    → asymmetric diffusion → pattern formation
spatial_autocorrelation(grid, d) → i8 → correlation at distance d
```

**Key types:**

- **`TernaryGrid`** — 2D grid of `i8` values in `{-1, 0, +1}`. Provides neighbor counting, majority voting, and discrete Laplacian computation.
- **`ReactionDiffusion`** — two coupled `TernaryGrid`s (activator and inhibitor) with ternary reaction parameters. The Gray-Scott-style update: activator grows where it's present but inhibited where inhibitor is strong; inhibitor grows in response to activator but is removed at rate `kill`.
- **`is_turing_unstable()`** — checks if diffusion rates differ (necessary condition for pattern formation).
- **`spatial_autocorrelation()`** — measures pattern regularity at a given distance.

## Usage

```rust
use ternary_morphogenesis::{TernaryGrid, ReactionDiffusion, is_turing_unstable, spatial_autocorrelation};

// Create a reaction-diffusion system
let mut rd = ReactionDiffusion::new(50, 50);

// Seed activator spots at specific locations
rd.seed_spots(&[(10, 10), (25, 25), (40, 40)]);

// Check if the system will form patterns
assert!(is_turing_unstable(&rd)); // da ≠ di → instability → patterns

// Run the reaction-diffusion simulation
rd.run(100);

// Check activator pattern
let center = rd.activator.get(25, 25);
println!("Center cell: {}", center); // should be non-zero if pattern formed

// Measure pattern diversity: count distinct 3×3 patches
let diversity = rd.pattern_diversity();
println!("Pattern diversity: {} distinct local patterns", diversity);

// Initialize stripes instead of spots
let mut rd_stripes = ReactionDiffusion::new(100, 50);
rd_stripes.seed_stripes(8); // 8-cell spacing
rd_stripes.run(50);

// Spatial autocorrelation: how correlated are cells at distance d?
let grid = TernaryGrid::new(10, 10);
let corr = spatial_autocorrelation(&grid, 1); // uniform grid → perfect correlation (= +1)

// Grid operations
let mut g = TernaryGrid::new(5, 5);
g.set(2, 2, 1);
let (neg, zero, pos) = g.neighbor_counts(2, 1); // count neighbors by state
let majority = g.majority(2, 1); // most common neighbor
let lap = g.laplacian(2, 2);    // discrete Laplacian, clamped to {-1, 0, +1}
```

## API Reference

### `TernaryGrid`

| Method | Description |
|--------|-------------|
| `TernaryGrid::new(width, height)` | Create grid initialized to 0 |
| `.get(x, y)` / `.set(x, y, v)` | Cell access (values clamped to `{-1, 0, +1}`) |
| `.neighbor_counts(x, y)` | Moore neighborhood: `(neg_count, zero_count, pos_count)` |
| `.majority(x, y)` | Most common neighbor value (ties → 0) |
| `.laplacian(x, y)` | `Σ(neighbors) - count × center`, clamped to ternary |

### `ReactionDiffusion`

| Method | Description |
|--------|-------------|
| `ReactionDiffusion::new(width, height)` | Create with default ternary parameters |
| `.seed_spots(&[(x, y), ...])` | Place 3×3 activator spots at given centers |
| `.seed_stripes(spacing)` | Initialize alternating stripe pattern |
| `.step()` | One reaction-diffusion iteration |
| `.run(steps)` | Run N iterations |
| `.pattern_diversity()` | Count distinct 3×3 patches in activator field |

### Free Functions

| Function | Description |
|----------|-------------|
| `is_turing_unstable(rd)` | True if `da ≠ di` and `feed ≠ 0` (asymmetric diffusion) |
| `spatial_autocorrelation(grid, distance)` | Ternary autocorrelation at given distance |

## The Deeper Idea

Turing's insight was that **instability creates order**. Two chemicals that are individually stable can, when coupled through diffusion, become collectively unstable. The activator produces more of itself (positive feedback) and also produces the inhibitor. The inhibitor suppresses the activator (negative feedback) but diffuses faster. The result: the activator forms localized peaks (spots) because it activates itself locally but is suppressed at a distance by the faster-diffusing inhibitor.

In the ternary version, this plays out through discrete state transitions. Each cell holds `{-1, 0, +1}` and updates based on its Laplacian (local neighborhood sum) and the reaction term (activator × inhibitor interaction). The ternary clamp after each update acts as a nonlinear threshold — small changes are absorbed (stay near 0), large changes are amplified (snap to ±1). This quantization introduces a new phenomenon not present in continuous reaction-diffusion: the system has **discrete basin boundaries** that continuous systems smooth over.

The `#![no_std]` constraint means this can run anywhere — microcontrollers, WASM, GPUs via compute shaders. Ternary state is compact (2 bits per cell) and the update rule is purely local (no global coordination needed). This makes it suitable for generating textures and patterns in constrained environments — game engines, embedded displays, procedural generation.

## Related Crates

- **`ternary-percolate`** — percolation theory on ternary grids, analyzing the connectivity of reaction-diffusion patterns
- **`ternary-renormalization`** — coarse-graining ternary fields, revealing the multi-scale structure of morphogenesis patterns
- **`ternary-scheduler`** — scheduling ternary tasks, where morphogenesis generates self-organizing task distributions
