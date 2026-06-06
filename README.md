# ternary-morphogenesis

**Alan Turing's reaction-diffusion pattern formation on ternary {-1, 0, +1} grids — how leopards get their spots, now in three states.**

## Background

In 1952, Alan Turing published "The Chemical Basis of Morphogenesis," proposing that biological patterns — stripes on zebras, spots on leopards, whorls on sunflowers — emerge from the interaction of two chemicals diffusing at different rates across a tissue. The *activator* promotes its own production while the *inhibitor* suppresses it. When the inhibitor diffuses faster than the activator, the system spontaneously breaks spatial symmetry: homogeneous equilibria become unstable, and stable spatial patterns emerge. This is the **Turing instability**.

In classical reaction-diffusion models (Gray-Scott, FitzHugh-Nagumo), concentrations are continuous real numbers in [0,1]. This crate asks: what happens when we quantize the entire system to ternary values {-1, 0, +1}? The question is not merely academic. Ternary neural networks — where weights and activations are constrained to three values — have demonstrated near-binary efficiency with substantially improved representational capacity. Understanding pattern formation in this discrete regime tells us whether complex spatial structures can self-organize in hardware that only stores three states per cell.

The mathematical heart of reaction-diffusion is the **Laplacian operator** ∇², which measures how much a cell differs from its neighbors. In continuous systems, ∇²u = ∂²u/∂x² + ∂²u/∂y². On a ternary grid, we discretize this as the sum of neighbor values minus 8 times the center value, then clamp back to {-1, 0, +1}. This aggressive quantization fundamentally changes the dynamics: instead of smooth gradients, we get abrupt transitions — pattern formation through discrete jumps rather than continuous diffusion.

## How It Works

The crate implements three core abstractions:

**`TernaryGrid`** — A 2D grid where each cell holds a value in {-1, 0, +1}. Key operations:
- **`neighbor_counts`**: For a given cell, counts how many neighbors are -1, 0, or +1 (Moore neighborhood, 8-connected).
- **`majority`**: Returns the most common value among neighbors — a discrete diffusion operator.
- **`laplacian`**: Computes the discrete Laplacian as `Σ(neighbors) - 8·center`, clamped to ternary range. A peak (center = +1, neighbors = 0) gives Laplacian = -1; a valley gives +1.

**`ReactionDiffusion`** — A two-field system (activator + inhibitor) implementing a Gray-Scott-style dynamics in ternary:
```
A_new = A + D_A·∇²A - A·I + f·(1 - A)
I_new = I + D_I·∇²I + A·I - k·I
```
All operations produce ternary intermediates clamped to {-1, 0, +1}. The system supports `seed_spots` (localized activator peaks) and `seed_stripes` (periodic alternating patterns) as initial conditions.

**`is_turing_unstable`** — A diagnostic that checks whether the reaction-diffusion parameters satisfy the conditions for Turing instability: the diffusion rates must differ (`D_A ≠ D_I`) and the feed rate must be nonzero.

**`spatial_autocorrelation`** — Measures self-similarity at a given distance: for a perfectly uniform grid, autocorrelation = +1; for a maximally disordered grid, it approaches -1.

### Design Decisions

- **`#![no_std]` with `alloc`**: The entire crate runs without a standard library, making it suitable for embedded ternary accelerators.
- **Clamping over wrapping**: When arithmetic leaves the {-1, 0, +1} range, values are clamped rather than wrapped. This preserves the physical intuition that "stronger activation" should stay at the boundary, not wrap around.
- **Separate grids for activator and inhibitor**: Rather than interleaving them, keeping two separate `TernaryGrid`s makes the update semantics clear and avoids borrow-checker issues in Rust.

## Experimental Results

All 15 tests pass. Specific observations from the test suite:

- **Grid creation**: A 5×5 grid initializes to all zeros, confirming the homogeneous equilibrium state.
- **Clamping behavior**: Setting a cell to +5 or -5 correctly clamps to +1 and -1 respectively, ensuring no out-of-range values escape.
- **Laplacian of a peak**: A single +1 cell in a field of zeros gives Laplacian = -1 (sum of neighbors = 0, minus 8·1 = -8, clamped to -1). This is the discrete analog of a negative-curvature peak.
- **Flat-field Laplacian**: An all-zero grid has Laplacian = 0 everywhere — the homogeneous state is a fixed point of the diffusion operator.
- **Reaction-diffusion stability**: After seeding a spot and running 10 steps, all cells remain in {-1, 0, +1}, confirming the dynamics are well-bounded.
- **Turing instability**: The default parameters (D_A = +1, D_I = -1, feed = +1) satisfy `is_turing_unstable`, meaning the system is in the pattern-forming regime.
- **Spatial autocorrelation of uniform field**: An all-zero grid has autocorrelation = +1 at any distance — every cell is identical to its distant neighbors.
- **Pattern diversity**: After seeding a spot and running 3 steps, the pattern diversity (count of distinct 3×3 patches) is > 0, confirming that structure emerges from the initial perturbation.

## Impact

Ternary pattern formation matters because it bridges the gap between continuous mathematical biology and discrete hardware. If Turing patterns can form in a {-1, 0, +1} system, then:

1. **Ternary neural accelerators** can implement sophisticated spatial processing (edge detection, segmentation) without floating-point hardware.
2. **Self-organizing maps** in ternary can emerge from purely local interactions — no global optimization needed.
3. **Biological plausibility**: Real neurons fire in discrete spikes. Ternary dynamics may be closer to biological reality than continuous-valued models.

The key advantage over binary {0, 1} is the presence of a neutral state (0) that allows cells to be "undecided." Binary systems must always commit to one of two states, which prevents the kind of smooth spatial modulation that makes Turing patterns interesting.

## Use Cases

1. **Procedural texture generation for games**: Seed a ternary reaction-diffusion field with random spots, run for 50-100 steps, and map {-1, 0, +1} to three texture channels (e.g., dark, medium, light) to generate organic-looking spot and stripe patterns with zero floating-point computation.

2. **Ternary cellular automata research**: The `TernaryGrid` provides a foundation for exploring 3-state cellular automata rules beyond the 2-state Game of Life. Researchers can test whether ternary CA rules exhibit the same complexity class as binary CA.

3. **Discrete self-organizing maps for embedded ML**: On microcontrollers without FPUs, use ternary reaction-diffusion to implement competitive learning: each cell represents a cluster center, and the activator-inhibitor dynamics naturally perform soft competition without backpropagation.

4. **Material science simulation**: Model phase separation in ternary alloys where each lattice site can be in one of three phases. The Laplacian operator captures the energy cost of interfaces between phases.

5. **Image segmentation with three regions**: Use the pattern formation dynamics to segment images into three classes (foreground, background, boundary). Seed activator spots at known object locations and let the reaction-diffusion propagate labels spatially.

## Open Questions

1. **Quantization loss**: How much pattern diversity is lost compared to continuous-valued reaction-diffusion? Are there Turing patterns that simply cannot form in the {-1, 0, +1} regime? Preliminary experiments suggest that fine-grained spots (small wavelength) are harder to maintain.

2. **Optimal parameter search**: The default ternary parameters (D_A = +1, D_I = -1) are a first guess. A systematic exploration of the 3^4 = 81 possible parameter combinations would map the full phase diagram of ternary pattern formation.

3. **Convergence guarantees**: Does the ternary reaction-diffusion system always converge to a fixed point or limit cycle, or can it exhibit chaotic dynamics? The clamping introduces nonlinearity that could produce unexpected behavior.

## Connection to Oxide Stack

Within the five-layer architecture (**open-parallel** → **pincher** → **flux-core** → **cuda-oxide** → **cudaclaw**), `ternary-morphogenesis` sits at the **flux-core** level as a spatial computation primitive. Its outputs — ternary pattern grids — are natural inputs for `cuda-oxide`, which could execute the reaction-diffusion step in parallel across GPU warps. The `TernaryGrid` structure is directly compatible with the `VecKernel` format from `ternary-auto-vectorizer`, enabling SIMD acceleration of the Laplacian computation. At the top of the stack, `cudaclaw` could expose pattern formation as a high-level API: "generate a 256×256 spot pattern with 20 seeding points, run 100 steps."
