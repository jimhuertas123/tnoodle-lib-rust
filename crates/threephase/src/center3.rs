#![allow(clippy::all)]
//! Port of cs.threephase.Center3
//!
//! Phase-3 center solver for the 4x4 three-phase algorithm.
//! Tracks three coordinate arrays (ud, rl, fb) encoding the relative
//! positions of center facelets within their face groups.
//!
//! Total states: 35 * 35 * 12 * 2 = 29400.
//! Move table has 20 moves (phase-3 move subset).

use std::sync::LazyLock;

use crate::center_cube::CenterCube;
use crate::util::{CNK, swap_i32};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Total number of phase-3 center coordinates.
pub const N_CT: usize = 35 * 35 * 12 * 2; // 29400

/// Number of phase-3 moves.
pub const N_MOVES: usize = 20;

/// Parity contribution of each of the 20 phase-3 moves.
const PMOVE: [i32; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1];

/// Maps the 12 valid RL combination indices to their standard C(8,4) indices.
const RL2STD: [i32; 12] = [0, 9, 14, 23, 27, 28, 41, 42, 46, 55, 60, 69];

// ---------------------------------------------------------------------------
// Tables (lazy-initialised)
// ---------------------------------------------------------------------------

pub static TABLES: LazyLock<Tables> = LazyLock::new(Tables::init);

pub struct Tables {
    /// Move table: `ctmove[coord][move] = coord'`.
    pub ctmove: Vec<[u16; N_MOVES]>,
    /// Pruning table.
    pub prun: Vec<i8>,
    /// Inverse map: standard C(8,4) index -> RL index (0..11), or 0 for invalid.
    pub std2rl: [i32; 70],
}

// ---------------------------------------------------------------------------
// Center3 working state
// ---------------------------------------------------------------------------

/// Represents a phase-3 center state.
#[derive(Clone, Debug)]
pub struct Center3 {
    pub ud: [i32; 8],
    pub rl: [i32; 8],
    pub fb: [i32; 8],
    pub parity: i32,
}

impl Default for Center3 {
    fn default() -> Self {
        Self::new()
    }
}

impl Center3 {
    pub fn new() -> Self {
        Self {
            ud: [0; 8],
            rl: [0; 8],
            fb: [0; 8],
            parity: 0,
        }
    }

    /// Set this Center3 from a CenterCube and an edge-corner parity value.
    pub fn set_from_cube(&mut self, c: &CenterCube, exc_parity: i32) {
        let a = (c.ct[0] % 3) as i32;
        let b = (c.ct[8] % 3) as i32;
        let d = (c.ct[16] % 3) as i32;
        let parity = if (a > b) ^ (b > d) ^ (a > d) { 0 } else { 1 };

        for i in 0..8 {
            self.ud[i] = ((c.ct[i] as i32 / 3) ^ 1) as i32;
            self.fb[i] = ((c.ct[i + 8] as i32 / 3) ^ 1) as i32;
            self.rl[i] = ((c.ct[i + 16] as i32 / 3) ^ 1 ^ parity) as i32;
        }
        self.parity = parity ^ exc_parity;
    }

    /// Encode the coordinate (0..29399).
    pub fn getct(&self, std2rl: &[i32; 70]) -> i32 {
        let mut idx = 0i32;
        let mut r = 4i32;
        for i in (0..=6).rev() {
            if self.ud[i] != self.ud[7] {
                idx += CNK[i][r as usize];
                r -= 1;
            }
        }

        idx *= 35;
        r = 4;
        for i in (0..=6).rev() {
            if self.fb[i] != self.fb[7] {
                idx += CNK[i][r as usize];
                r -= 1;
            }
        }

        idx *= 12;
        let check = self.fb[7] ^ self.ud[7];
        let mut idxrl = 0i32;
        r = 4;
        for i in (0..=7).rev() {
            if self.rl[i] != check {
                idxrl += CNK[i][r as usize];
                r -= 1;
            }
        }

        self.parity + 2 * (idx + std2rl[idxrl as usize])
    }

    /// Decode the coordinate.
    pub fn setct(&mut self, mut idx: i32) {
        self.parity = idx & 1;
        idx >>= 1;

        let idxrl_local = RL2STD[(idx % 12) as usize];
        idx /= 12;

        // Decode RL
        let mut r = 4i32;
        let mut idxrl = idxrl_local;
        for i in (0..=7).rev() {
            self.rl[i] = 0;
            if idxrl >= CNK[i][r as usize] {
                idxrl -= CNK[i][r as usize];
                r -= 1;
                self.rl[i] = 1;
            }
        }

        // Decode FB
        let mut idxfb = idx % 35;
        idx /= 35;
        r = 4;
        self.fb[7] = 0;
        for i in (0..=6).rev() {
            if idxfb >= CNK[i][r as usize] {
                idxfb -= CNK[i][r as usize];
                r -= 1;
                self.fb[i] = 1;
            } else {
                self.fb[i] = 0;
            }
        }

        // Decode UD
        r = 4;
        self.ud[7] = 0;
        for i in (0..=6).rev() {
            if idx >= CNK[i][r as usize] {
                idx -= CNK[i][r as usize];
                r -= 1;
                self.ud[i] = 1;
            } else {
                self.ud[i] = 0;
            }
        }
    }

    /// Apply a phase-3 move (0..19).
    pub fn apply_move(&mut self, i: usize) {
        self.parity ^= PMOVE[i];
        match i {
            0 | 1 | 2 => {
                // U, U2, U'
                swap_i32(&mut self.ud, 0, 1, 2, 3, (i % 3) as i32);
            }
            3 => {
                // R2
                swap_i32(&mut self.rl, 0, 1, 2, 3, 1);
            }
            4 | 5 | 6 => {
                // F, F2, F'
                swap_i32(&mut self.fb, 0, 1, 2, 3, ((i - 1) % 3) as i32);
            }
            7 | 8 | 9 => {
                // D, D2, D'
                swap_i32(&mut self.ud, 4, 5, 6, 7, ((i - 1) % 3) as i32);
            }
            10 => {
                // L2
                swap_i32(&mut self.rl, 4, 5, 6, 7, 1);
            }
            11 | 12 | 13 => {
                // B, B2, B'
                swap_i32(&mut self.fb, 4, 5, 6, 7, ((i + 1) % 3) as i32);
            }
            14 => {
                // u2
                swap_i32(&mut self.ud, 0, 1, 2, 3, 1);
                swap_i32(&mut self.rl, 0, 5, 4, 1, 1);
                swap_i32(&mut self.fb, 0, 5, 4, 1, 1);
            }
            15 => {
                // r2
                swap_i32(&mut self.rl, 0, 1, 2, 3, 1);
                swap_i32(&mut self.fb, 1, 4, 7, 2, 1);
                swap_i32(&mut self.ud, 1, 6, 5, 2, 1);
            }
            16 => {
                // f2
                swap_i32(&mut self.fb, 0, 1, 2, 3, 1);
                swap_i32(&mut self.ud, 3, 2, 5, 4, 1);
                swap_i32(&mut self.rl, 0, 3, 6, 5, 1);
            }
            17 => {
                // d2
                swap_i32(&mut self.ud, 4, 5, 6, 7, 1);
                swap_i32(&mut self.rl, 3, 2, 7, 6, 1);
                swap_i32(&mut self.fb, 3, 2, 7, 6, 1);
            }
            18 => {
                // l2
                swap_i32(&mut self.rl, 4, 5, 6, 7, 1);
                swap_i32(&mut self.fb, 0, 3, 6, 5, 1);
                swap_i32(&mut self.ud, 0, 3, 4, 7, 1);
            }
            19 => {
                // b2
                swap_i32(&mut self.fb, 4, 5, 6, 7, 1);
                swap_i32(&mut self.ud, 0, 7, 6, 1, 1);
                swap_i32(&mut self.rl, 1, 4, 7, 2, 1);
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Table initialisation
// ---------------------------------------------------------------------------

impl Tables {
    fn init() -> Self {
        // Build std2rl inverse mapping
        let mut std2rl = [0i32; 70];
        for i in 0..12 {
            std2rl[RL2STD[i] as usize] = i as i32;
        }

        let mut tables = Tables {
            ctmove: vec![[0u16; N_MOVES]; N_CT],
            prun: vec![-1i8; N_CT],
            std2rl,
        };

        // Build move table
        let mut c = Center3::new();
        for i in 0..N_CT {
            for m in 0..N_MOVES {
                c.setct(i as i32);
                c.apply_move(m);
                tables.ctmove[i][m] = c.getct(&tables.std2rl) as u16;
            }
        }

        // Build pruning table via BFS
        tables.prun[0] = 0;
        let mut depth: i8 = 0;
        let mut done = 1usize;

        while done != N_CT {
            for i in 0..N_CT {
                if tables.prun[i] != depth {
                    continue;
                }
                for m in 0..17 {
                    let next = tables.ctmove[i][m] as usize;
                    if tables.prun[next] == -1 {
                        tables.prun[next] = depth + 1;
                        done += 1;
                    }
                }
            }
            depth += 1;
        }

        tables
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tables_initialize() {
        let tables = &*TABLES;
        // Pruning table should be fully filled
        assert!(tables.prun.iter().all(|&v| v >= 0));
        // Root should be 0
        assert_eq!(tables.prun[0], 0);
        // Move table should have correct dimensions
        assert_eq!(tables.ctmove.len(), N_CT);
    }

    #[test]
    fn test_setct_getct_roundtrip() {
        let tables = &*TABLES;
        let mut c = Center3::new();
        for idx in [0, 1, 100, 1000, 10000, 29399] {
            c.setct(idx);
            let got = c.getct(&tables.std2rl);
            assert_eq!(got, idx, "set/get roundtrip failed for idx={}", idx);
        }
    }

    #[test]
    fn test_std2rl_mapping() {
        let tables = &*TABLES;
        // Verify std2rl is the inverse of rl2std for valid entries
        for i in 0..12 {
            let std_idx = RL2STD[i] as usize;
            assert_eq!(
                tables.std2rl[std_idx], i as i32,
                "std2rl[{}] should be {} but got {}",
                std_idx, i, tables.std2rl[std_idx]
            );
        }
    }

    #[test]
    fn test_move_table_populated() {
        let tables = &*TABLES;
        // At least one non-zero entry
        let has_nonzero = tables.ctmove.iter().any(|row| row.iter().any(|&v| v != 0));
        assert!(has_nonzero, "move table should have non-zero entries");
    }
}
