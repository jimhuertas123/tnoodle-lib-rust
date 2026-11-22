#![allow(clippy::all)]
//! Port of cs.threephase.Center2
//!
//! Phase-2 center solver for the 4x4 three-phase algorithm.
//! Tracks two coordinates:
//!   - `rl`: 8 R/L center positions (C(7,4)*2 = 70 states with parity)
//!   - `ct`: 16 U/D/F/B center positions (C(15,8) = 6435 states)
//! Pruning table has 6435 * 70 = 450450 entries.

use std::sync::LazyLock;

use crate::center_cube::CenterCube;
use crate::moves;
use crate::util::{CNK, swap_i32};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of RL coordinate states (C(7,4) * 2 = 70).
pub const N_RL: usize = 70;

/// Number of CT coordinate states (C(15,8) = 6435).
pub const N_CT: usize = 6435;

/// Size of the pruning table.
pub const N_PRUN: usize = N_CT * N_RL; // 450450

// ---------------------------------------------------------------------------
// Tables (lazy-initialised)
// ---------------------------------------------------------------------------

pub static TABLES: LazyLock<Tables> = LazyLock::new(Tables::init);

pub struct Tables {
    /// RL move table: `rlmv[rl_coord][move] = rl_coord'`.
    pub rlmv: Vec<[i32; 28]>,
    /// CT move table: `ctmv[ct_coord][move] = ct_coord'`.
    pub ctmv: Vec<[u16; 28]>,
    /// RL rotation table: `rlrot[rl_coord][rot] = rl_coord'`.
    pub rlrot: Vec<[i32; 16]>,
    /// CT rotation table: `ctrot[ct_coord][rot] = ct_coord'`.
    pub ctrot: Vec<[u16; 16]>,
    /// Pruning table indexed by `ct_coord * 70 + rl_coord`.
    pub ctprun: Vec<i8>,
}

/// Parity contribution per standard move (36 entries, indexed 0..35).
const PMV: [i32; 36] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0,
    1, 0, 0, 0,
];

// ---------------------------------------------------------------------------
// Center2 working state
// ---------------------------------------------------------------------------

/// Represents a phase-2 center state.
#[derive(Clone, Debug)]
pub struct Center2 {
    /// R/L centers (8 elements).
    pub rl: [i32; 8],
    /// U/D/F/B centers (16 elements).
    pub ct: [i32; 16],
    /// Edge parity bit.
    pub parity: i32,
}

impl Default for Center2 {
    fn default() -> Self {
        Self::new()
    }
}

impl Center2 {
    pub fn new() -> Self {
        Self {
            rl: [0; 8],
            ct: [0; 16],
            parity: 0,
        }
    }

    /// Set this Center2 from a CenterCube and an edge parity value.
    pub fn set(&mut self, c: &CenterCube, edge_parity: i32) {
        for i in 0..16 {
            self.ct[i] = (c.ct[i] % 3) as i32;
        }
        for i in 0..8 {
            self.rl[i] = c.ct[i + 16] as i32;
        }
        self.parity = edge_parity;
    }

    /// Encode the RL coordinate.
    pub fn getrl(&self) -> i32 {
        let mut idx = 0i32;
        let mut r = 4i32;
        for i in (0..=6).rev() {
            if self.rl[i] != self.rl[7] {
                idx += CNK[i][r as usize];
                r -= 1;
            }
        }
        idx * 2 + self.parity
    }

    /// Decode the RL coordinate.
    pub fn setrl(&mut self, mut idx: i32) {
        self.parity = idx & 1;
        idx >>= 1;
        let mut r = 4i32;
        self.rl[7] = 0;
        for i in (0..=6).rev() {
            if idx >= CNK[i][r as usize] {
                idx -= CNK[i][r as usize];
                r -= 1;
                self.rl[i] = 1;
            } else {
                self.rl[i] = 0;
            }
        }
    }

    /// Encode the CT coordinate.
    pub fn getct(&self) -> i32 {
        let mut idx = 0i32;
        let mut r = 8i32;
        for i in (0..=14).rev() {
            if self.ct[i] != self.ct[15] {
                idx += CNK[i][r as usize];
                r -= 1;
            }
        }
        idx
    }

    /// Decode the CT coordinate.
    pub fn setct(&mut self, mut idx: i32) {
        let mut r = 8i32;
        self.ct[15] = 0;
        for i in (0..=14).rev() {
            if idx >= CNK[i][r as usize] {
                idx -= CNK[i][r as usize];
                r -= 1;
                self.ct[i] = 1;
            } else {
                self.ct[i] = 0;
            }
        }
    }

    /// Apply a rotation.
    pub fn rot(&mut self, r: usize) {
        match r {
            0 => {
                self.apply_move(moves::UWX2 as usize);
                self.apply_move(moves::DWX2 as usize);
            }
            1 => {
                self.apply_move(moves::RWX1 as usize);
                self.apply_move(moves::LWX3 as usize);
            }
            2 => {
                swap_i32(&mut self.ct, 0, 3, 1, 2, 1);
                swap_i32(&mut self.ct, 8, 11, 9, 10, 1);
                swap_i32(&mut self.ct, 4, 7, 5, 6, 1);
                swap_i32(&mut self.ct, 12, 15, 13, 14, 1);
                swap_i32(&mut self.rl, 0, 3, 5, 6, 1);
                swap_i32(&mut self.rl, 1, 2, 4, 7, 1);
            }
            _ => {}
        }
    }

    /// Apply a single move (0..35, standard move index).
    pub fn apply_move(&mut self, m: usize) {
        self.parity ^= PMV[m];
        let key = (m % 3) as i32;
        let face = m / 3;
        match face {
            0 => {
                // U
                swap_i32(&mut self.ct, 0, 1, 2, 3, key);
            }
            1 => {
                // R
                swap_i32(&mut self.rl, 0, 1, 2, 3, key);
            }
            2 => {
                // F
                swap_i32(&mut self.ct, 8, 9, 10, 11, key);
            }
            3 => {
                // D
                swap_i32(&mut self.ct, 4, 5, 6, 7, key);
            }
            4 => {
                // L
                swap_i32(&mut self.rl, 4, 5, 6, 7, key);
            }
            5 => {
                // B
                swap_i32(&mut self.ct, 12, 13, 14, 15, key);
            }
            6 => {
                // u
                swap_i32(&mut self.ct, 0, 1, 2, 3, key);
                swap_i32(&mut self.rl, 0, 5, 4, 1, key);
                swap_i32(&mut self.ct, 8, 9, 12, 13, key);
            }
            7 => {
                // r
                swap_i32(&mut self.rl, 0, 1, 2, 3, key);
                swap_i32(&mut self.ct, 1, 15, 5, 9, key);
                swap_i32(&mut self.ct, 2, 12, 6, 10, key);
            }
            8 => {
                // f
                swap_i32(&mut self.ct, 8, 9, 10, 11, key);
                swap_i32(&mut self.rl, 0, 3, 6, 5, key);
                swap_i32(&mut self.ct, 3, 2, 5, 4, key);
            }
            9 => {
                // d
                swap_i32(&mut self.ct, 4, 5, 6, 7, key);
                swap_i32(&mut self.rl, 3, 2, 7, 6, key);
                swap_i32(&mut self.ct, 11, 10, 15, 14, key);
            }
            10 => {
                // l
                swap_i32(&mut self.rl, 4, 5, 6, 7, key);
                swap_i32(&mut self.ct, 0, 8, 4, 14, key);
                swap_i32(&mut self.ct, 3, 11, 7, 13, key);
            }
            11 => {
                // b
                swap_i32(&mut self.ct, 12, 13, 14, 15, key);
                swap_i32(&mut self.rl, 1, 4, 7, 2, key);
                swap_i32(&mut self.ct, 1, 0, 7, 6, key);
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
        let mut tables = Tables {
            rlmv: vec![[0i32; 28]; N_RL],
            ctmv: vec![[0u16; 28]; N_CT],
            rlrot: vec![[0i32; 16]; N_RL],
            ctrot: vec![[0u16; 16]; N_CT],
            ctprun: vec![-1i8; N_PRUN],
        };

        let mut c = Center2::new();

        // RL move table
        for i in 0..(N_RL as i32) {
            for m in 0..28 {
                c.setrl(i);
                c.apply_move(moves::MOVE2STD[m] as usize);
                tables.rlmv[i as usize][m] = c.getrl();
            }
        }

        // RL rotation table
        for i in 0..(N_RL as i32) {
            c.setrl(i);
            for j in 0..16 {
                tables.rlrot[i as usize][j] = c.getrl();
                c.rot(0);
                if j % 2 == 1 {
                    c.rot(1);
                }
                if j % 8 == 7 {
                    c.rot(2);
                }
            }
        }

        // CT rotation table
        for i in 0..(N_CT as i32) {
            c.setct(i);
            for j in 0..16 {
                tables.ctrot[i as usize][j] = c.getct() as u16;
                c.rot(0);
                if j % 2 == 1 {
                    c.rot(1);
                }
                if j % 8 == 7 {
                    c.rot(2);
                }
            }
        }

        // CT move table
        for i in 0..(N_CT as i32) {
            for m in 0..28 {
                c.setct(i);
                c.apply_move(moves::MOVE2STD[m] as usize);
                tables.ctmv[i as usize][m] = c.getct() as u16;
            }
        }

        // Pruning table: BFS from the 6 solved states
        tables.ctprun[0] = 0;
        tables.ctprun[18] = 0;
        tables.ctprun[28] = 0;
        tables.ctprun[46] = 0;
        tables.ctprun[54] = 0;
        tables.ctprun[56] = 0;
        let mut depth: i8 = 0;
        let mut done = 6usize;

        while done != N_PRUN {
            for i in 0..N_PRUN {
                if tables.ctprun[i] != depth {
                    continue;
                }
                let ct_idx = i / N_RL;
                let rl_idx = i % N_RL;
                for m in 0..23 {
                    let ctx = tables.ctmv[ct_idx][m] as usize;
                    let rlx = tables.rlmv[rl_idx][m] as usize;
                    let idx = ctx * N_RL + rlx;
                    if tables.ctprun[idx] == -1 {
                        tables.ctprun[idx] = depth + 1;
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
        // Check pruning table is fully filled
        assert!(tables.ctprun.iter().all(|&v| v >= 0));
        // Check pruning root entries
        assert_eq!(tables.ctprun[0], 0);
        assert_eq!(tables.ctprun[18], 0);
        assert_eq!(tables.ctprun[28], 0);
        assert_eq!(tables.ctprun[46], 0);
        assert_eq!(tables.ctprun[54], 0);
        assert_eq!(tables.ctprun[56], 0);
    }

    #[test]
    fn test_rl_set_get_roundtrip() {
        let mut c = Center2::new();
        for idx in 0..N_RL as i32 {
            c.setrl(idx);
            assert_eq!(c.getrl(), idx, "rl roundtrip failed for idx={}", idx);
        }
    }

    #[test]
    fn test_ct_set_get_roundtrip() {
        let mut c = Center2::new();
        for idx in [0, 1, 100, 1000, 6434] {
            c.setct(idx);
            assert_eq!(c.getct(), idx, "ct roundtrip failed for idx={}", idx);
        }
    }

    #[test]
    fn test_move_tables_populated() {
        let tables = &*TABLES;
        // At least one non-zero entry in RL move table
        let has_nonzero = tables.rlmv.iter().any(|row| row.iter().any(|&v| v != 0));
        assert!(has_nonzero, "rl move table should have non-zero entries");
        // At least one non-zero entry in CT move table
        let has_nonzero = tables.ctmv.iter().any(|row| row.iter().any(|&v| v != 0));
        assert!(has_nonzero, "ct move table should have non-zero entries");
    }
}
