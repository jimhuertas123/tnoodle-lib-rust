#![allow(clippy::all)]
//! Port of cs.threephase.Center1
//!
//! Phase-1 center solver for the 4x4 three-phase algorithm.
//! Uses symmetry reduction with 48 symmetries to compress
//! 735471 raw states down to 15582 symmetry-reduced states.

use std::sync::LazyLock;

use crate::center_cube::CenterCube;
use crate::full_cube::CENTER_FACELET;
use crate::moves;
use crate::util::{swap_u8, CNK};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of symmetry-reduced center-1 coordinates.
pub const N_SYM: usize = 15582;

/// Total number of raw center-1 states (C(24,8) = 735471).
pub const N_RAW: usize = 735471;

/// Number of symmetries (full octahedral group).
pub const N_SYM_GROUP: usize = 48;

// ---------------------------------------------------------------------------
// Tables (lazy-initialised)
// ---------------------------------------------------------------------------

pub static TABLES: LazyLock<Tables> = LazyLock::new(Tables::init);

pub struct Tables {
    /// Move table: `ctsmv[sym_coord][move] = sym_coord'` (encoded with sym in low 6 bits).
    pub ctsmv: Vec<[u32; 36]>,
    /// Maps symmetry-reduced index -> raw index.
    pub sym2raw: Vec<i32>,
    /// Pruning table (depth for each sym coordinate).
    pub csprun: Vec<i8>,

    /// Symmetry multiplication table: `symmult[i][j] = k` means sym_i * sym_j = sym_k.
    pub symmult: [[i32; N_SYM_GROUP]; N_SYM_GROUP],
    /// Symmetry-conjugated move table: `symmove[sym][move] = move'`.
    pub symmove: [[i32; 36]; N_SYM_GROUP],
    /// Inverse symmetry: `syminv[i]` is the index of the inverse of symmetry `i`.
    pub syminv: [i32; N_SYM_GROUP],
    /// Finish (solved state under each inverse symmetry).
    pub finish: [i32; N_SYM_GROUP],

    /// Maps raw index -> (sym_reduced_index << 6 | sym).
    pub raw2sym: Vec<i32>,
}

// ---------------------------------------------------------------------------
// Center1 working state
// ---------------------------------------------------------------------------

/// Represents a center-1 state: a 24-element binary array indicating
/// which centers belong to one particular face-colour class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Center1 {
    pub ct: [u8; 24],
}

impl Default for Center1 {
    fn default() -> Self {
        Self::new()
    }
}

impl Center1 {
    /// Solved state: first 8 centers = 1, rest = 0.
    pub fn new() -> Self {
        let mut ct = [0u8; 24];
        for i in 0..8 {
            ct[i] = 1;
        }
        Self { ct }
    }

    /// Construct from a raw byte array.
    pub fn from_ct(ct: [u8; 24]) -> Self {
        Self { ct }
    }

    /// Construct from a CenterCube, extracting binary mask for one URF class.
    pub fn from_center_cube(c: &CenterCube, urf: u8) -> Self {
        let mut ct = [0u8; 24];
        for i in 0..24 {
            ct[i] = if c.ct[i] % 3 == urf { 1 } else { 0 };
        }
        Self { ct }
    }

    /// Apply a single move (0..35).
    pub fn apply_move(&mut self, m: usize) {
        let key = (m % 3) as i32;
        let face = m / 3;
        match face {
            0 => {
                // U
                swap_u8(&mut self.ct, 0, 1, 2, 3, key);
            }
            1 => {
                // R
                swap_u8(&mut self.ct, 16, 17, 18, 19, key);
            }
            2 => {
                // F
                swap_u8(&mut self.ct, 8, 9, 10, 11, key);
            }
            3 => {
                // D
                swap_u8(&mut self.ct, 4, 5, 6, 7, key);
            }
            4 => {
                // L
                swap_u8(&mut self.ct, 20, 21, 22, 23, key);
            }
            5 => {
                // B
                swap_u8(&mut self.ct, 12, 13, 14, 15, key);
            }
            6 => {
                // u
                swap_u8(&mut self.ct, 0, 1, 2, 3, key);
                swap_u8(&mut self.ct, 8, 20, 12, 16, key);
                swap_u8(&mut self.ct, 9, 21, 13, 17, key);
            }
            7 => {
                // r
                swap_u8(&mut self.ct, 16, 17, 18, 19, key);
                swap_u8(&mut self.ct, 1, 15, 5, 9, key);
                swap_u8(&mut self.ct, 2, 12, 6, 10, key);
            }
            8 => {
                // f
                swap_u8(&mut self.ct, 8, 9, 10, 11, key);
                swap_u8(&mut self.ct, 2, 19, 4, 21, key);
                swap_u8(&mut self.ct, 3, 16, 5, 22, key);
            }
            9 => {
                // d
                swap_u8(&mut self.ct, 4, 5, 6, 7, key);
                swap_u8(&mut self.ct, 10, 18, 14, 22, key);
                swap_u8(&mut self.ct, 11, 19, 15, 23, key);
            }
            10 => {
                // l
                swap_u8(&mut self.ct, 20, 21, 22, 23, key);
                swap_u8(&mut self.ct, 0, 8, 4, 14, key);
                swap_u8(&mut self.ct, 3, 11, 7, 13, key);
            }
            11 => {
                // b
                swap_u8(&mut self.ct, 12, 13, 14, 15, key);
                swap_u8(&mut self.ct, 1, 20, 7, 18, key);
                swap_u8(&mut self.ct, 0, 23, 6, 17, key);
            }
            _ => {}
        }
    }

    /// Decode a raw combinatorial index into the ct array.
    pub fn set(&mut self, mut idx: i32) {
        let mut r = 8i32;
        for i in (0..24).rev() {
            self.ct[i] = 0;
            if idx >= CNK[i][r as usize] {
                idx -= CNK[i][r as usize];
                r -= 1;
                self.ct[i] = 1;
            }
        }
    }

    /// Encode the ct array into a raw combinatorial index.
    pub fn get(&self) -> i32 {
        let mut idx = 0i32;
        let mut r = 8i32;
        for i in (0..24).rev() {
            if self.ct[i] == 1 {
                idx += CNK[i][r as usize];
                r -= 1;
            }
        }
        idx
    }

    /// Copy state from another Center1.
    pub fn copy_from(&mut self, other: &Center1) {
        self.ct.copy_from_slice(&other.ct);
    }

    /// Apply one of the four generating rotations.
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
                swap_u8(&mut self.ct, 0, 3, 1, 2, 1);
                swap_u8(&mut self.ct, 8, 11, 9, 10, 1);
                swap_u8(&mut self.ct, 4, 7, 5, 6, 1);
                swap_u8(&mut self.ct, 12, 15, 13, 14, 1);
                swap_u8(&mut self.ct, 16, 19, 21, 22, 1);
                swap_u8(&mut self.ct, 17, 18, 20, 23, 1);
            }
            3 => {
                self.apply_move(moves::UWX1 as usize);
                self.apply_move(moves::DWX3 as usize);
                self.apply_move(moves::FWX1 as usize);
                self.apply_move(moves::BWX3 as usize);
            }
            _ => {}
        }
    }

    /// Apply the composite rotation indexed by `r` (0..47).
    pub fn rotate(&mut self, r: usize) {
        for j in 0..r {
            self.rot(0);
            if j % 2 == 1 {
                self.rot(1);
            }
            if j % 8 == 7 {
                self.rot(2);
            }
            if j % 16 == 15 {
                self.rot(3);
            }
        }
    }

    /// Get the symmetry-reduced coordinate using the raw2sym lookup table.
    /// Returns (sym_idx << 6 | symmetry).
    pub fn getsym(&self, tables: &Tables) -> i32 {
        tables.raw2sym[self.get() as usize]
    }

    /// Binary search for a raw index in the sym2raw table.
    /// Returns the sym-reduced index or -1 if not found.
    pub fn raw2sym_search(sym2raw: &[i32], n: i32) -> i32 {
        match sym2raw.binary_search(&n) {
            Ok(m) => m as i32,
            Err(_) => -1,
        }
    }

    /// Determine which symmetry maps a solved CenterCube to its current state.
    /// Returns symmetry index 0..47, or -1 if none found.
    pub fn get_solved_sym(cube: &CenterCube) -> i32 {
        let mut c = Center1::from_ct(cube.ct);
        for j in 0..48 {
            let mut check = true;
            for i in 0..24 {
                if c.ct[i] != CENTER_FACELET[i] / 16 {
                    check = false;
                    break;
                }
            }
            if check {
                return j as i32;
            }
            c.rot(0);
            if j % 2 == 1 {
                c.rot(1);
            }
            if j % 8 == 7 {
                c.rot(2);
            }
            if j % 16 == 15 {
                c.rot(3);
            }
        }
        -1
    }
}

/// Rotation-to-string table (sparse -- many entries are empty).
pub const ROT2STR: [&str; 48] = [
    "",       "y2",     "x",      "x y2",   "x2",     "z2",     "x'",     "x' y2",
    "",       "",       "",       "",       "",       "",       "",       "",
    "y z",    "y' z'",  "y2 z",   "z'",     "y' z",   "y z'",   "z",      "z y2",
    "",       "",       "",       "",       "",       "",       "",       "",
    "y' x'",  "y x",    "y'",     "y",      "y' x",   "y x'",   "y z2",   "y' z2",
    "",       "",       "",       "",       "",       "",       "",       "",
];

// ---------------------------------------------------------------------------
// Table initialisation
// ---------------------------------------------------------------------------

impl Tables {
    fn init() -> Self {
        let mut tables = Tables {
            ctsmv: vec![[0u32; 36]; N_SYM],
            sym2raw: vec![0i32; N_SYM],
            csprun: vec![-1i8; N_SYM],

            symmult: [[0i32; N_SYM_GROUP]; N_SYM_GROUP],
            symmove: [[0i32; 36]; N_SYM_GROUP],
            syminv: [0i32; N_SYM_GROUP],
            finish: [0i32; N_SYM_GROUP],

            raw2sym: vec![0i32; N_RAW],
        };

        tables.init_sym();
        tables.init_sym2raw();
        tables.create_move_table();
        tables.create_prun();

        tables
    }

    /// Initialise symmetry tables: symmult, symmove, syminv, finish.
    fn init_sym(&mut self) {
        // Start with identity-like state [0,1,2,...,23]
        let mut c = Center1 { ct: [0u8; 24] };
        for i in 0..24u8 {
            c.ct[i as usize] = i;
        }
        let d_init = c.clone();
        let e_init = c.clone();

        let mut d = d_init.clone();

        // Compute symmult and syminv
        for i in 0..48 {
            for j in 0..48 {
                d.copy_from(&d_init);
                for k in 0..48 {
                    if c == d {
                        self.symmult[i][j] = k as i32;
                        if k == 0 {
                            self.syminv[i] = j as i32;
                        }
                    }
                    d.rot(0);
                    if k % 2 == 1 {
                        d.rot(1);
                    }
                    if k % 8 == 7 {
                        d.rot(2);
                    }
                    if k % 16 == 15 {
                        d.rot(3);
                    }
                }
                c.rot(0);
                if j % 2 == 1 {
                    c.rot(1);
                }
                if j % 8 == 7 {
                    c.rot(2);
                }
                if j % 16 == 15 {
                    c.rot(3);
                }
            }
            c.rot(0);
            if i % 2 == 1 {
                c.rot(1);
            }
            if i % 8 == 7 {
                c.rot(2);
            }
            if i % 16 == 15 {
                c.rot(3);
            }
        }

        // Compute symmove: symmove[i][j] = sym_i^-1 * move_j * sym_i
        let mut f = e_init.clone();
        for i in 0..48 {
            c.copy_from(&e_init);
            c.rotate(self.syminv[i] as usize);
            for j in 0..36 {
                d.copy_from(&c);
                d.apply_move(j);
                d.rotate(i);
                for k in 0..36 {
                    f.copy_from(&e_init);
                    f.apply_move(k);
                    if f == d {
                        self.symmove[i][j] = k as i32;
                        break;
                    }
                }
            }
        }

        // Compute finish: solved state under each inverse symmetry
        c.set(0);
        for i in 0..48 {
            self.finish[self.syminv[i] as usize] = c.get();
            c.rot(0);
            if i % 2 == 1 {
                c.rot(1);
            }
            if i % 8 == 7 {
                c.rot(2);
            }
            if i % 16 == 15 {
                c.rot(3);
            }
        }
    }

    /// Initialise sym2raw and raw2sym tables.
    fn init_sym2raw(&mut self) {
        let mut c = Center1::new();
        let mut occ = vec![0u32; N_RAW / 32 + 1];
        let mut count = 0usize;

        for i in 0..N_RAW {
            if (occ[i >> 5] & (1 << (i & 0x1f))) == 0 {
                c.set(i as i32);
                for j in 0..48 {
                    let idx = c.get() as usize;
                    occ[idx >> 5] |= 1 << (idx & 0x1f);
                    self.raw2sym[idx] = (count as i32) << 6 | self.syminv[j];
                    c.rot(0);
                    if j % 2 == 1 {
                        c.rot(1);
                    }
                    if j % 8 == 7 {
                        c.rot(2);
                    }
                    if j % 16 == 15 {
                        c.rot(3);
                    }
                }
                self.sym2raw[count] = i as i32;
                count += 1;
            }
        }
        assert_eq!(count, N_SYM);
    }

    /// Create the symmetry-reduced move table.
    fn create_move_table(&mut self) {
        let mut c = Center1::new();
        let mut d = Center1::new();

        for i in 0..N_SYM {
            d.set(self.sym2raw[i]);
            for m in 0..36 {
                c.copy_from(&d);
                c.apply_move(m);
                self.ctsmv[i][m] = self.raw2sym[c.get() as usize] as u32;
            }
        }
    }

    /// Create the pruning table using BFS.
    fn create_prun(&mut self) {
        self.csprun[0] = 0;
        let mut depth: i8 = 0;
        let mut done = 1usize;

        while done != N_SYM {
            let inv = depth > 4;
            let select: i8 = if inv { -1 } else { depth };
            let check: i8 = if inv { depth } else { -1 };
            depth += 1;

            for i in 0..N_SYM {
                if self.csprun[i] != select {
                    continue;
                }
                for m in 0..27 {
                    let idx = (self.ctsmv[i][m] >> 6) as usize;
                    if self.csprun[idx] != check {
                        continue;
                    }
                    done += 1;
                    if inv {
                        self.csprun[i] = depth;
                        break;
                    } else {
                        self.csprun[idx] = depth;
                    }
                }
            }
        }
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
        // Check sym2raw has the right number of entries
        assert_eq!(tables.sym2raw.len(), N_SYM);
        // First entry should be 0 (identity)
        assert_eq!(tables.sym2raw[0], 0);
        // Pruning table should be fully filled (no -1 remaining)
        assert!(tables.csprun.iter().all(|&v| v >= 0));
        // Pruning root should be 0
        assert_eq!(tables.csprun[0], 0);
    }

    #[test]
    fn test_center1_set_get_roundtrip() {
        let mut c = Center1::new();
        for idx in [0, 1, 100, 1000, 10000, 735470] {
            c.set(idx);
            assert_eq!(c.get(), idx, "set/get roundtrip failed for idx={}", idx);
        }
    }

    #[test]
    fn test_syminv_is_inverse() {
        let tables = &*TABLES;
        for i in 0..N_SYM_GROUP {
            let inv = tables.syminv[i] as usize;
            assert_eq!(
                tables.symmult[i][inv], 0,
                "syminv[{}] = {} but symmult[{}][{}] = {} (expected 0)",
                i, inv, i, inv, tables.symmult[i][inv]
            );
        }
    }

    #[test]
    fn test_move_table_populated() {
        let tables = &*TABLES;
        // At least one non-zero entry in the move table
        let has_nonzero = tables.ctsmv.iter().any(|row| row.iter().any(|&v| v != 0));
        assert!(has_nonzero, "move table should have non-zero entries");
    }
}
