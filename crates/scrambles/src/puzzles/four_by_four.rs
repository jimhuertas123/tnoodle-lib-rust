//! 4x4x4 scramble generation.
//!
//! Java's `FourByFourCubePuzzle` uses random-state scrambles from the threephase
//! solver. The Rust threephase port does not produce valid solutions yet, so
//! 4x4x4 uses canonicalized random turns (Java's `444fast`) for now.

use rand::Rng;

use super::{big_cubes, cube_puzzle::CubePuzzle};
use crate::puzzle::{Puzzle, solve_in};

/// Generate a 4x4x4 scramble (random turns, 40 moves).
pub fn generate_scramble(rng: &mut impl Rng) -> String {
    big_cubes::generate_scramble(4, rng)
}

/// Generate a 4x4x4 blindfolded scramble: the scramble followed by a random
/// whole-cube rotation such as `x y2`. Port of `NoInspectionFourByFourCubePuzzle`.
pub fn generate_scramble_ni(rng: &mut impl Rng) -> String {
    let puzzle = CubePuzzle::new(4);
    loop {
        let psag = puzzle.generate_with_orientation(rng, puzzle.size() - 1, false);
        if solve_in(&puzzle, &psag.state, puzzle.wca_min_scramble_distance() - 1).is_none() {
            return psag.generator;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_444_scramble() {
        let mut rng = rand::rng();
        let scramble = generate_scramble(&mut rng);
        assert_eq!(scramble.split_whitespace().count(), 40);
    }

    #[test]
    fn generate_444ni_scramble() {
        let mut rng = rand::rng();
        for _ in 0..10 {
            let scramble = generate_scramble_ni(&mut rng);
            let moves: Vec<&str> = scramble.split_whitespace().collect();
            let rotations = moves.iter().filter(|m| m.starts_with(['x', 'y', 'z'])).count();
            assert!(rotations <= 2 && moves.len() == 40 + rotations, "{scramble}");
        }
    }
}
