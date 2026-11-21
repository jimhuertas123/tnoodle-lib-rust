#![allow(clippy::all)]
//! Port of cs.threephase.EdgeCube
//!
//! Represents the 24 edge positions of a 4×4×4 Rubik's cube.
//! Edges come in 12 pairs (two stickers each), so there are 24 edge slots.

use rand::Rng;

use crate::moves;
use crate::util::{parity, swap_u8};

/// Edge colour mapping: `EDGE_COLOR[i]` = [face0, face1] for edge i (0..11).
pub const EDGE_COLOR: [[usize; 2]; 12] = [
    [moves::F_FACE, moves::U_FACE],
    [moves::L_FACE, moves::U_FACE],
    [moves::B_FACE, moves::U_FACE],
    [moves::R_FACE, moves::U_FACE],
    [moves::B_FACE, moves::D_FACE],
    [moves::L_FACE, moves::D_FACE],
    [moves::F_FACE, moves::D_FACE],
    [moves::R_FACE, moves::D_FACE],
    [moves::F_FACE, moves::L_FACE],
    [moves::B_FACE, moves::L_FACE],
    [moves::B_FACE, moves::R_FACE],
    [moves::F_FACE, moves::R_FACE],
];

/// Edge facelet map: maps the 24 edge positions to 54-facelet indices.
pub const EDGE_MAP: [u8; 24] = [
    moves::F2,
    moves::L2,
    moves::B2,
    moves::R2,
    moves::B8,
    moves::L8,
    moves::F8,
    moves::R8,
    moves::F4,
    moves::B6,
    moves::B4,
    moves::F6,
    moves::U8,
    moves::U4,
    moves::U2,
    moves::U6,
    moves::D8,
    moves::D4,
    moves::D2,
    moves::D6,
    moves::L6,
    moves::L4,
    moves::R6,
    moves::R4,
];

/// 4×4×4 edge cube state: 24 edge positions, each storing a value 0..23.
#[derive(Clone, Debug)]
pub struct EdgeCube {
    pub ep: [u8; 24],
}

impl Default for EdgeCube {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeCube {
    /// Create a solved edge cube (identity permutation).
    #[must_use]
    pub fn new() -> Self {
        let mut ep = [0u8; 24];
        for i in 0..24 {
            ep[i] = i as u8;
        }
        Self { ep }
    }

    /// Create a random edge permutation.
    pub fn new_random(rng: &mut impl Rng) -> Self {
        let mut cube = Self::new();
        for i in 0..23 {
            let t = i + rng.random_range(0..(24 - i));
            if t != i {
                cube.ep.swap(i, t);
            }
        }
        cube
    }

    /// Create an edge cube by applying a move sequence to the solved state.
    pub fn from_moves(moveseq: &[usize]) -> Self {
        let mut cube = Self::new();
        for &m in moveseq {
            cube.apply_move(m);
        }
        cube
    }

    /// Get the parity of the edge permutation (0 = even, 1 = odd).
    pub fn get_parity(&self) -> i32 {
        parity(&self.ep)
    }

    /// Copy state from another `EdgeCube`.
    pub fn copy_from(&mut self, other: &EdgeCube) {
        self.ep.copy_from_slice(&other.ep);
    }

    /// Fill edge facelets of a 3×3×3 facelet array (54 chars).
    pub fn fill_333_facelet(&self, facelet: &mut [u8; 54]) {
        let faces = b"URFDLB";
        for i in 0..24 {
            let ep_i = self.ep[i] as usize;
            facelet[EDGE_MAP[i] as usize] = faces[EDGE_COLOR[ep_i % 12][ep_i / 12]];
        }
    }

    /// Validate the edge state.
    ///
    /// Checks that positions 0..11 contain exactly one representative from
    /// each of the 12 edge pairs (no duplicates) and that the parity of the
    /// split between the two halves is even.
    ///
    /// Returns `true` if the edge state is valid.
    pub fn check_edge(&self) -> bool {
        let mut ck: i32 = 0;
        let mut par = false;
        for i in 0..12 {
            ck |= 1 << self.ep[i];
            par ^= self.ep[i] >= 12;
        }
        ck &= ck >> 12;
        ck == 0 && !par
    }

    /// Apply a single move to this edge cube.
    ///
    /// `m` is a move index 0..35:
    ///   - m / 3 gives the face (0=U, 1=R, 2=F, 3=D, 4=L, 5=B,
    ///     6=u, 7=r, 8=f, 9=d, 10=l, 11=b)
    ///   - m % 3 gives the twist amount (0 = CW, 1 = 180°, 2 = CCW)
    pub fn apply_move(&mut self, m: usize) {
        let key = (m % 3) as i32;
        let face = m / 3;
        match face {
            0 => {
                // U
                swap_u8(&mut self.ep, 0, 1, 2, 3, key);
                swap_u8(&mut self.ep, 12, 13, 14, 15, key);
            }
            1 => {
                // R
                swap_u8(&mut self.ep, 11, 15, 10, 19, key);
                swap_u8(&mut self.ep, 23, 3, 22, 7, key);
            }
            2 => {
                // F
                swap_u8(&mut self.ep, 0, 11, 6, 8, key);
                swap_u8(&mut self.ep, 12, 23, 18, 20, key);
            }
            3 => {
                // D
                swap_u8(&mut self.ep, 4, 5, 6, 7, key);
                swap_u8(&mut self.ep, 16, 17, 18, 19, key);
            }
            4 => {
                // L
                swap_u8(&mut self.ep, 1, 20, 5, 21, key);
                swap_u8(&mut self.ep, 13, 8, 17, 9, key);
            }
            5 => {
                // B
                swap_u8(&mut self.ep, 2, 9, 4, 10, key);
                swap_u8(&mut self.ep, 14, 21, 16, 22, key);
            }
            6 => {
                // u
                swap_u8(&mut self.ep, 0, 1, 2, 3, key);
                swap_u8(&mut self.ep, 12, 13, 14, 15, key);
                swap_u8(&mut self.ep, 9, 22, 11, 20, key);
            }
            7 => {
                // r
                swap_u8(&mut self.ep, 11, 15, 10, 19, key);
                swap_u8(&mut self.ep, 23, 3, 22, 7, key);
                swap_u8(&mut self.ep, 2, 16, 6, 12, key);
            }
            8 => {
                // f
                swap_u8(&mut self.ep, 0, 11, 6, 8, key);
                swap_u8(&mut self.ep, 12, 23, 18, 20, key);
                swap_u8(&mut self.ep, 3, 19, 5, 13, key);
            }
            9 => {
                // d
                swap_u8(&mut self.ep, 4, 5, 6, 7, key);
                swap_u8(&mut self.ep, 16, 17, 18, 19, key);
                swap_u8(&mut self.ep, 8, 23, 10, 21, key);
            }
            10 => {
                // l
                swap_u8(&mut self.ep, 1, 20, 5, 21, key);
                swap_u8(&mut self.ep, 13, 8, 17, 9, key);
                swap_u8(&mut self.ep, 14, 0, 18, 4, key);
            }
            11 => {
                // b
                swap_u8(&mut self.ep, 2, 9, 4, 10, key);
                swap_u8(&mut self.ep, 14, 21, 16, 22, key);
                swap_u8(&mut self.ep, 7, 15, 1, 17, key);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let cube = EdgeCube::new();
        for i in 0..24 {
            assert_eq!(cube.ep[i], i as u8);
        }
    }

    #[test]
    fn test_identity_parity() {
        let cube = EdgeCube::new();
        assert_eq!(cube.get_parity(), 0);
    }

    #[test]
    fn test_check_edge_solved() {
        let cube = EdgeCube::new();
        assert!(cube.check_edge());
    }

    #[test]
    fn test_u4_identity() {
        // U applied 4 times should return to identity
        let identity = EdgeCube::new();
        let mut cube = EdgeCube::new();
        for _ in 0..4 {
            cube.apply_move(0); // U (CW)
        }
        assert_eq!(cube.ep, identity.ep);
    }

    #[test]
    fn test_u2_twice_identity() {
        // U2 applied twice should return to identity
        let identity = EdgeCube::new();
        let mut cube = EdgeCube::new();
        for _ in 0..2 {
            cube.apply_move(1); // U2
        }
        assert_eq!(cube.ep, identity.ep);
    }

    #[test]
    fn test_u_prime_reverses_u() {
        let identity = EdgeCube::new();
        let mut cube = EdgeCube::new();
        cube.apply_move(0); // U
        cube.apply_move(2); // U'
        assert_eq!(cube.ep, identity.ep);
    }

    #[test]
    fn test_r4_identity() {
        let identity = EdgeCube::new();
        let mut cube = EdgeCube::new();
        for _ in 0..4 {
            cube.apply_move(3); // R (CW)
        }
        assert_eq!(cube.ep, identity.ep);
    }

    #[test]
    fn test_wide_u4_identity() {
        // u (wide U, m=18 => face=6) applied 4 times
        let identity = EdgeCube::new();
        let mut cube = EdgeCube::new();
        for _ in 0..4 {
            cube.apply_move(18); // u CW
        }
        assert_eq!(cube.ep, identity.ep);
    }

    #[test]
    fn test_from_moves_empty() {
        let cube = EdgeCube::from_moves(&[]);
        let solved = EdgeCube::new();
        assert_eq!(cube.ep, solved.ep);
    }

    #[test]
    fn test_copy_from() {
        let mut a = EdgeCube::new();
        a.apply_move(0);
        let mut b = EdgeCube::new();
        b.copy_from(&a);
        assert_eq!(a.ep, b.ep);
    }

    #[test]
    fn test_single_move_changes_state() {
        let identity = EdgeCube::new();
        let mut cube = EdgeCube::new();
        cube.apply_move(0); // U
        assert_ne!(cube.ep, identity.ep);
    }
}
