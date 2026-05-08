#![allow(clippy::all)]
//! Port of cs.threephase.CornerCube
//!
//! Represents the 8 corner cubies of a 4×4×4 Rubik's cube.
//! Each corner has a permutation index (`cp`) and orientation (`co`).

use std::sync::LazyLock;

use rand::Rng;

use crate::moves;
use crate::util::{parity, set8_perm};

/// 18 precomputed move cubes, initialised lazily.
static MOVE_CUBE: LazyLock<[CornerCube; 18]> = LazyLock::new(init_move);

/// Corner facelet indices (using the 54-facelet 3×3×3 numbering from Moves).
///
/// `CORNER_FACELET[c]` = [U/D facelet, second facelet, third facelet]
/// for corner `c`, following the Java `cornerFacelet` array in CornerCube.java.
pub const CORNER_FACELET: [[u8; 3]; 8] = [
    [moves::U9, moves::R1, moves::F3],
    [moves::U7, moves::F1, moves::L3],
    [moves::U1, moves::L1, moves::B3],
    [moves::U3, moves::B1, moves::R3],
    [moves::D3, moves::F9, moves::R7],
    [moves::D1, moves::L9, moves::F7],
    [moves::D7, moves::B9, moves::L7],
    [moves::D9, moves::R9, moves::B7],
];

/// 4×4×4 corner cube state.
#[derive(Clone, Debug)]
pub struct CornerCube {
    /// Corner permutation (8 elements, values 0..7).
    pub cp: [u8; 8],
    /// Corner orientation (8 elements, values 0..2).
    pub co: [u8; 8],
}

impl Default for CornerCube {
    fn default() -> Self {
        Self::new()
    }
}

impl CornerCube {
    /// Create a solved corner cube (identity permutation, zero twist).
    #[must_use]
    pub fn new() -> Self {
        Self {
            cp: [0, 1, 2, 3, 4, 5, 6, 7],
            co: [0, 0, 0, 0, 0, 0, 0, 0],
        }
    }

    /// Create a random corner cube.
    pub fn new_random(rng: &mut impl Rng) -> Self {
        let cperm = rng.random_range(0..40320);
        let twist = rng.random_range(0..2187);
        Self::from_cperm_twist(cperm, twist)
    }

    /// Create a corner cube from a permutation index and twist index.
    #[must_use]
    pub fn from_cperm_twist(cperm: i32, twist: i32) -> Self {
        let mut cube = Self::new();
        cube.set_c_perm(cperm);
        cube.set_twist(twist);
        cube
    }

    /// Copy state from another `CornerCube`.
    pub fn copy_from(&mut self, other: &CornerCube) {
        self.cp.copy_from_slice(&other.cp);
        self.co.copy_from_slice(&other.co);
    }

    /// Get the parity of the corner permutation (0 = even, 1 = odd).
    pub fn get_parity(&self) -> i32 {
        parity(&self.cp)
    }

    /// Fill corner facelets of a 3×3×3 facelet array (54 chars).
    pub fn fill_333_facelet(&self, facelet: &mut [u8; 54]) {
        let faces = b"URFDLB";
        for corn in 0..8 {
            let j = self.cp[corn] as usize;
            let ori = self.co[corn] as usize;
            for n in 0..3 {
                facelet[CORNER_FACELET[corn][(n + ori) % 3] as usize] =
                    faces[(CORNER_FACELET[j][n] / 9) as usize];
            }
        }
    }

    /// Corner multiplication: `prod = a * b`.
    ///
    /// Computes the product of two corner cubes `a` and `b`, storing the
    /// result in `prod`. This corresponds to applying `b` first, then `a`.
    pub fn corn_mult(a: &CornerCube, b: &CornerCube, prod: &mut CornerCube) {
        for corn in 0..8 {
            prod.cp[corn] = a.cp[b.cp[corn] as usize];
            let ori_a = a.co[b.cp[corn] as usize];
            let ori_b = b.co[corn];
            let mut ori = ori_a;
            ori += if ori_a < 3 { ori_b } else { 6 - ori_b };
            ori %= 3;
            if (ori_a >= 3) ^ (ori_b >= 3) {
                ori += 3;
            }
            prod.co[corn] = ori;
        }
    }

    /// Set the corner twist from an index (0..2186).
    pub fn set_twist(&mut self, mut idx: i32) {
        let mut twst: i32 = 0;
        for i in (0..=6).rev() {
            let val = (idx % 3) as u8;
            self.co[i] = val;
            twst += val as i32;
            idx /= 3;
        }
        self.co[7] = ((15 - twst) % 3) as u8;
    }

    /// Set the corner permutation from an index (0..40319).
    pub fn set_c_perm(&mut self, idx: i32) {
        set8_perm(&mut self.cp, idx);
    }

    /// Apply a single move to this corner cube.
    ///
    /// `idx` is a move index 0..17 (the 18 outer-layer moves).
    pub fn apply_move(&mut self, idx: usize) {
        let mc = &MOVE_CUBE[idx];
        let mut temp = CornerCube::new();
        Self::corn_mult(self, mc, &mut temp);
        self.copy_from(&temp);
    }
}

/// Initialise the 18 move cubes (6 faces × 3 twist amounts).
fn init_move() -> [CornerCube; 18] {
    let mut cubes: [CornerCube; 18] = std::array::from_fn(|_| CornerCube::new());

    // Base moves (CW quarter turn) for each of the 6 faces
    cubes[0] = CornerCube::from_cperm_twist(15120, 0);
    cubes[3] = CornerCube::from_cperm_twist(21021, 1494);
    cubes[6] = CornerCube::from_cperm_twist(8064, 1236);
    cubes[9] = CornerCube::from_cperm_twist(9, 0);
    cubes[12] = CornerCube::from_cperm_twist(1230, 412);
    cubes[15] = CornerCube::from_cperm_twist(224, 137);

    // Derive the half-turn and inverse-quarter-turn for each face
    for a in (0..18).step_by(3) {
        for p in 0..2 {
            let mut prod = CornerCube::new();
            CornerCube::corn_mult(&cubes[a + p], &cubes[a], &mut prod);
            cubes[a + p + 1] = prod;
        }
    }

    cubes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let cube = CornerCube::new();
        assert_eq!(cube.cp, [0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(cube.co, [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_identity_parity() {
        let cube = CornerCube::new();
        assert_eq!(cube.get_parity(), 0);
    }

    #[test]
    fn test_u4_identity() {
        // U applied 4 times should return to identity
        let identity = CornerCube::new();
        let mut cube = CornerCube::new();
        for _ in 0..4 {
            cube.apply_move(0); // U (CW)
        }
        assert_eq!(cube.cp, identity.cp);
        assert_eq!(cube.co, identity.co);
    }

    #[test]
    fn test_u2_is_double_u() {
        // U2 should equal U applied twice
        let mut cube_u2 = CornerCube::new();
        cube_u2.apply_move(1); // U2

        let mut cube_uu = CornerCube::new();
        cube_uu.apply_move(0); // U
        cube_uu.apply_move(0); // U

        assert_eq!(cube_u2.cp, cube_uu.cp);
        assert_eq!(cube_u2.co, cube_uu.co);
    }

    #[test]
    fn test_u_prime_reverses_u() {
        let identity = CornerCube::new();
        let mut cube = CornerCube::new();
        cube.apply_move(0); // U
        cube.apply_move(2); // U'
        assert_eq!(cube.cp, identity.cp);
        assert_eq!(cube.co, identity.co);
    }

    #[test]
    fn test_from_cperm_twist_identity() {
        let cube = CornerCube::from_cperm_twist(0, 0);
        assert_eq!(cube.cp, [0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(cube.co, [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_corn_mult_identity() {
        let a = CornerCube::new();
        let b = CornerCube::new();
        let mut prod = CornerCube::new();
        CornerCube::corn_mult(&a, &b, &mut prod);
        assert_eq!(prod.cp, [0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(prod.co, [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_copy_from() {
        let a = CornerCube::from_cperm_twist(15120, 0);
        let mut b = CornerCube::new();
        b.copy_from(&a);
        assert_eq!(a.cp, b.cp);
        assert_eq!(a.co, b.co);
    }

    #[test]
    fn test_move_cubes_initialized() {
        // Just force lazy init and verify the base move is not identity
        let mc = &MOVE_CUBE[0]; // U
        assert_ne!(mc.cp, [0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_set_twist_roundtrip() {
        // Twist 0 should give all-zero orientation
        let mut cube = CornerCube::new();
        cube.set_twist(0);
        assert_eq!(cube.co, [0, 0, 0, 0, 0, 0, 0, 0]);

        // The sum of orientations must always be divisible by 3
        cube.set_twist(1234);
        let sum: u8 = cube.co.iter().sum();
        assert_eq!(sum % 3, 0);
    }
}
