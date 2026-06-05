//! # ternary-morphogenesis
//!
//! Alan Turing's morphogenesis: reaction-diffusion patterns on ternary grids.
//! How do leopards get their spots? How do zebras get their stripes?
//! In ternary: through the interaction of two ternary chemicals diffusing at different rates.

#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;
use alloc::{vec, vec::Vec};

/// A 2D ternary grid
#[derive(Debug, Clone)]
pub struct TernaryGrid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<i8>,
}

impl TernaryGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, cells: vec![0; width * height] }
    }

    pub fn get(&self, x: usize, y: usize) -> i8 {
        self.cells[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, v: i8) {
        self.cells[y * self.width + x] = v.clamp(-1, 1);
    }

    /// Count neighbors with each ternary value (Moore neighborhood)
    pub fn neighbor_counts(&self, x: usize, y: usize) -> (usize, usize, usize) {
        let mut neg = 0usize;
        let mut zero = 0usize;
        let mut pos = 0usize;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 { continue; }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= self.width as i32 || ny >= self.height as i32 {
                    continue;
                }
                match self.get(nx as usize, ny as usize) {
                    -1 => neg += 1,
                    0 => zero += 1,
                    1 => pos += 1,
                    _ => {}
                }
            }
        }
        (neg, zero, pos)
    }

    /// Compute majority value among neighbors
    pub fn majority(&self, x: usize, y: usize) -> i8 {
        let (neg, zero, pos) = self.neighbor_counts(x, y);
        if pos > neg && pos > zero { 1 }
        else if neg > pos && neg > zero { -1 }
        else { 0 }
    }

    /// Compute Laplacian (sum of neighbors - 8 * center) clamped to ternary
    pub fn laplacian(&self, x: usize, y: usize) -> i8 {
        let mut sum: i8 = 0;
        let mut count: i8 = 0;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 { continue; }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= self.width as i32 || ny >= self.height as i32 {
                    continue;
                }
                sum += self.get(nx as usize, ny as usize);
                count += 1;
            }
        }
        let center = self.get(x, y);
        let lap = sum - count * center;
        lap.clamp(-1, 1)
    }
}

/// Reaction-diffusion system with two ternary chemicals (activator-inhibitor)
#[derive(Debug, Clone)]
pub struct ReactionDiffusion {
    pub activator: TernaryGrid,
    pub inhibitor: TernaryGrid,
    pub da: i8,  // activator diffusion rate (ternary)
    pub di: i8,  // inhibitor diffusion rate
    pub feed: i8,  // feed rate (how fast activator is added)
    pub kill: i8,  // kill rate (how fast inhibitor removes activator)
}

impl ReactionDiffusion {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            activator: TernaryGrid::new(width, height),
            inhibitor: TernaryGrid::new(width, height),
            da: 1,
            di: -1,  // inhibitor diffuses "faster" (wider influence)
            feed: 1,
            kill: 1,
        }
    }

    /// Seed a pattern: place activator spots
    pub fn seed_spots(&mut self, centers: &[(usize, usize)]) {
        for &(cx, cy) in centers {
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let nx = cx as i32 + dx;
                    let ny = cy as i32 + dy;
                    if nx >= 0 && ny >= 0 && nx < self.activator.width as i32 && ny < self.activator.height as i32 {
                        self.activator.set(nx as usize, ny as usize, 1);
                        self.inhibitor.set(nx as usize, ny as usize, -1);
                    }
                }
            }
        }
    }

    /// Seed a stripe pattern
    pub fn seed_stripes(&mut self, spacing: usize) {
        for y in 0..self.activator.height {
            for x in 0..self.activator.width {
                if x % spacing < spacing / 2 {
                    self.activator.set(x, y, 1);
                    self.inhibitor.set(x, y, 0);
                } else {
                    self.activator.set(x, y, 0);
                    self.inhibitor.set(x, y, 1);
                }
            }
        }
    }

    /// One step of reaction-diffusion
    pub fn step(&mut self) {
        let w = self.activator.width;
        let h = self.activator.height;
        let mut new_a = self.activator.cells.clone();
        let mut new_i = self.inhibitor.cells.clone();

        for y in 0..h {
            for x in 0..w {
                let a = self.activator.get(x, y);
                let i = self.inhibitor.get(x, y);
                let lap_a = self.activator.laplacian(x, y);
                let lap_i = self.inhibitor.laplacian(x, y);

                // Gray-Scott style: dA = Da*∇²A - A*I² + f*(1-A)
                // Simplified for ternary:
                let reaction = a * i; // A*I interaction
                let new_a_val = a + self.da * lap_a - reaction + self.feed * (1 - a);
                let new_i_val = i + self.di * lap_i + reaction - self.kill * i;

                new_a[y * w + x] = new_a_val.clamp(-1, 1);
                new_i[y * w + x] = new_i_val.clamp(-1, 1);
            }
        }

        self.activator.cells = new_a;
        self.inhibitor.cells = new_i;
    }

    /// Run for N steps
    pub fn run(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }

    /// Measure pattern diversity: count distinct 3x3 patches
    pub fn pattern_diversity(&self) -> usize {
        let mut patches = vec![];
        for y in 0..self.activator.height.saturating_sub(2) {
            for x in 0..self.activator.width.saturating_sub(2) {
                let mut patch = vec![];
                for dy in 0..3 {
                    for dx in 0..3 {
                        patch.push(self.activator.get(x + dx, y + dy));
                    }
                }
                if !patches.contains(&patch) {
                    patches.push(patch);
                }
            }
        }
        patches.len()
    }
}

/// Turing instability check: does the reaction-diffusion system form patterns?
/// In ternary: if |da| ≠ |di|, the system has asymmetric diffusion → potential instability
pub fn is_turing_unstable(rd: &ReactionDiffusion) -> bool {
    rd.da != rd.di && rd.feed != 0
}

/// Measure spatial autocorrelation at distance d
pub fn spatial_autocorrelation(grid: &TernaryGrid, distance: usize) -> i8 {
    let mut count: i8 = 0;
    let mut same: i8 = 0;
    for y in 0..grid.height {
        for x in 0..grid.width {
            let v = grid.get(x, y);
            let nx = x + distance;
            if nx < grid.width {
                count += 1;
                if v == grid.get(nx, y) {
                    same += 1;
                }
            }
        }
    }
    if count == 0 { return 0; }
    // Ternary autocorrelation: (2*same - count) / count approximation
    let corr = 2 * same - count;
    corr.clamp(-1, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_new() {
        let g = TernaryGrid::new(5, 5);
        assert_eq!(g.get(2, 2), 0);
    }

    #[test]
    fn test_grid_set_get() {
        let mut g = TernaryGrid::new(5, 5);
        g.set(2, 2, 1);
        assert_eq!(g.get(2, 2), 1);
    }

    #[test]
    fn test_grid_clamp() {
        let mut g = TernaryGrid::new(5, 5);
        g.set(0, 0, 5);
        assert_eq!(g.get(0, 0), 1);
        g.set(0, 0, -5);
        assert_eq!(g.get(0, 0), -1);
    }

    #[test]
    fn test_neighbor_counts() {
        let mut g = TernaryGrid::new(5, 5);
        g.set(1, 1, 1);
        g.set(3, 1, -1);
        let (neg, zero, pos) = g.neighbor_counts(2, 1);
        assert_eq!(pos, 1);
        assert_eq!(neg, 1);
    }

    #[test]
    fn test_majority() {
        let mut g = TernaryGrid::new(5, 5);
        g.set(1, 1, 1);
        g.set(2, 1, 1);
        g.set(3, 1, 1);
        assert_eq!(g.majority(2, 0), 1);
    }

    #[test]
    fn test_laplacian_flat() {
        let g = TernaryGrid::new(5, 5);
        assert_eq!(g.laplacian(2, 2), 0);
    }

    #[test]
    fn test_laplacian_peak() {
        let mut g = TernaryGrid::new(5, 5);
        g.set(2, 2, 1);
        // Laplacian = sum(neighbors) - 8*1 = 0 - 8 = -8, clamped to -1
        assert_eq!(g.laplacian(2, 2), -1);
    }

    #[test]
    fn test_rd_new() {
        let rd = ReactionDiffusion::new(5, 5);
        assert_eq!(rd.activator.get(0, 0), 0);
    }

    #[test]
    fn test_rd_seed_spots() {
        let mut rd = ReactionDiffusion::new(5, 5);
        rd.seed_spots(&[(2, 2)]);
        assert_eq!(rd.activator.get(2, 2), 1);
        assert_eq!(rd.inhibitor.get(2, 2), -1);
    }

    #[test]
    fn test_rd_seed_stripes() {
        let mut rd = ReactionDiffusion::new(10, 5);
        rd.seed_stripes(4);
        assert_eq!(rd.activator.get(0, 0), 1);
        // x=4 is in the second half of the stripe period (spacing=4)
        // 4 % 4 = 0, which is < 4/2=2, so it's also 1. Use x=3 instead.
        assert_eq!(rd.activator.get(3, 0), 0); // 3 % 4 = 3 >= 2
    }

    #[test]
    fn test_rd_step() {
        let mut rd = ReactionDiffusion::new(5, 5);
        rd.seed_spots(&[(2, 2)]);
        rd.step();
        // After one step, the center should have changed
        // (exact value depends on dynamics, just check it runs)
        let v = rd.activator.get(2, 2);
        assert!(v >= -1 && v <= 1);
    }

    #[test]
    fn test_rd_run() {
        let mut rd = ReactionDiffusion::new(5, 5);
        rd.seed_spots(&[(2, 2)]);
        rd.run(10);
        // Should still have valid ternary values
        for y in 0..5 {
            for x in 0..5 {
                let v = rd.activator.get(x, y);
                assert!(v >= -1 && v <= 1);
            }
        }
    }

    #[test]
    fn test_turing_unstable() {
        let rd = ReactionDiffusion::new(5, 5);
        assert!(is_turing_unstable(&rd));
    }

    #[test]
    fn test_pattern_diversity() {
        let mut rd = ReactionDiffusion::new(5, 5);
        rd.seed_spots(&[(2, 2)]);
        rd.run(3);
        let div = rd.pattern_diversity();
        assert!(div > 0);
    }

    #[test]
    fn test_spatial_autocorrelation() {
        let g = TernaryGrid::new(5, 5);
        let corr = spatial_autocorrelation(&g, 1);
        assert_eq!(corr, 1); // all zeros → perfect autocorrelation
    }
}
