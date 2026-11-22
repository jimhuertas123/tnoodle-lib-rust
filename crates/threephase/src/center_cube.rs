#![allow(clippy::all)]
//! Port of cs.threephase.CenterCube
//!
//! Represents the 24 center facelets of a 4×4×4 Rubik's cube.
//! Each center position stores a face index (0..5).

use tnoodle_random::JavaRandom;

use crate::full_cube::CENTER_FACELET;
use crate::util::swap_u8;

/// Map from logical center index to 3×3×3 face for `fill_333_facelet`.
const CENTER_333_MAP: [usize; 6] = [0, 4, 2, 1, 5, 3];

/// 4×4×4 center cube state: 24 center facelets, each storing a face value 0..5.
#[derive(Clone, Debug)]
pub struct CenterCube {
    pub ct: [u8; 24],
}

impl Default for CenterCube {
    fn default() -> Self {
        Self::new()
    }
}

impl CenterCube {
    /// Create a solved center cube.
    ///
    /// Each center is initialised from `CENTER_FACELET[i] / 16`, matching the
    /// Java `FullCube.centerFacelet[i] / 16`.
    #[must_use]
    pub fn new() -> Self {
        let mut ct = [0u8; 24];
        for i in 0..24 {
            ct[i] = (CENTER_FACELET[i] / 16) as u8;
        }
        Self { ct }
    }

    /// Create a random center permutation.
    pub fn new_random(rng: &mut JavaRandom) -> Self {
        let mut cube = Self::new();
        for i in 0..23 {
            let t = i + rng.next_index(24 - i);
            if cube.ct[t] != cube.ct[i] {
                cube.ct.swap(i, t);
            }
        }
        cube
    }

    /// Create a center cube by applying a move sequence (indices into the
    /// 36-move table) to the solved state.
    #[must_use]
    pub fn from_moves(moveseq: &[usize]) -> Self {
        let mut cube = Self::new();
        for &m in moveseq {
            cube.apply_move(m);
        }
        cube
    }

    /// Copy state from another `CenterCube`.
    pub fn copy_from(&mut self, other: &CenterCube) {
        self.ct.copy_from_slice(&other.ct);
    }

    /// Fill the center facelets of a 3×3×3 facelet array (54 chars).
    ///
    /// # Panics
    ///
    /// If the four centers of a face are not identical (an unsolved center
    /// group).
    pub fn fill_333_facelet(&self, facelet: &mut [u8; 54]) {
        let first_idx = 4usize;
        let inc = 9usize;
        let faces = b"URFDLB";
        for i in 0..6 {
            let idx = CENTER_333_MAP[i] << 2;
            if self.ct[idx] != self.ct[idx + 1]
                || self.ct[idx + 1] != self.ct[idx + 2]
                || self.ct[idx + 2] != self.ct[idx + 3]
            {
                panic!("Unsolved Center");
            }
            facelet[first_idx + i * inc] = faces[self.ct[idx] as usize];
        }
    }

    /// Apply a single move to this center cube.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let cube = CenterCube::new();
        // Solved state: each group of 4 consecutive centers shares the same face
        for i in 0..6 {
            let base = i * 4;
            let face_val = cube.ct[base];
            for j in 0..4 {
                assert_eq!(cube.ct[base + j], face_val);
            }
        }
    }

    #[test]
    fn test_u4_identity() {
        // Applying U (m=0) four times should return to identity
        let original = CenterCube::new();
        let mut cube = CenterCube::new();
        for _ in 0..4 {
            cube.apply_move(0); // U (CW)
        }
        assert_eq!(cube.ct, original.ct);
    }

    #[test]
    fn test_u2_twice_identity() {
        // Applying U2 (m=1) twice should return to identity
        let original = CenterCube::new();
        let mut cube = CenterCube::new();
        for _ in 0..2 {
            cube.apply_move(1); // U2
        }
        assert_eq!(cube.ct, original.ct);
    }

    #[test]
    fn test_u_prime_reverses_u() {
        let original = CenterCube::new();
        let mut cube = CenterCube::new();
        cube.apply_move(0); // U
        cube.apply_move(2); // U'
        assert_eq!(cube.ct, original.ct);
    }

    #[test]
    fn test_wide_u4_identity() {
        // u (wide U, m=18 => face=6) applied 4 times
        let original = CenterCube::new();
        let mut cube = CenterCube::new();
        for _ in 0..4 {
            cube.apply_move(18); // u CW
        }
        assert_eq!(cube.ct, original.ct);
    }

    #[test]
    fn test_from_moves_empty() {
        let cube = CenterCube::from_moves(&[]);
        let solved = CenterCube::new();
        assert_eq!(cube.ct, solved.ct);
    }

    #[test]
    fn test_copy_from() {
        let a = CenterCube::new();
        let mut b = CenterCube::new();
        b.ct[0] = 99;
        b.copy_from(&a);
        assert_eq!(a.ct, b.ct);
    }
}
