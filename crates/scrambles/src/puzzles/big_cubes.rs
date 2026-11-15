//! Random-turn scramble generation for 4x4x4 (fast), 5x5x5, 6x6x6, 7x7x7 cubes.
//!
//! Uses the generic random-turn generator with `AlgorithmBuilder`, exactly like
//! Java's `CubePuzzle`: a move is rejected when it would cancel or merge with an
//! earlier move it commutes with (so no `R R`, `R L R`, `Rw Lw ... Rw`, ...).

use rand::Rng;

use super::cube_puzzle::CubePuzzle;
use crate::puzzle::{Puzzle, solve_in};

/// Generate a WCA random-turn scramble for an NxN cube.
pub fn generate_scramble(size: usize, rng: &mut (impl Rng + ?Sized)) -> String {
    CubePuzzle::new(size).generate_wca_scramble(rng)
}

/// Generate a 5x5x5 blindfolded scramble: random turns followed by a random
/// reorientation such as `3Rw 3Uw'`. Port of `NoInspectionFiveByFiveCubePuzzle`.
pub fn generate_scramble_555ni(rng: &mut (impl Rng + ?Sized)) -> String {
    let puzzle = CubePuzzle::new(5);
    loop {
        let psag = puzzle.generate_with_orientation(rng, puzzle.size() / 2, true);
        if solve_in(&puzzle, &psag.state, puzzle.wca_min_scramble_distance() - 1).is_none() {
            return psag.generator;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scramble_lengths_match_java() {
        let mut rng = rand::rng();
        for (size, len) in [(4, 40), (5, 60), (6, 80), (7, 100)] {
            let scramble = generate_scramble(size, &mut rng);
            assert_eq!(scramble.split_whitespace().count(), len, "{scramble}");
        }
    }

    #[test]
    fn generate_555ni_scramble() {
        let mut rng = rand::rng();
        for _ in 0..10 {
            let scramble = generate_scramble_555ni(&mut rng);
            let moves: Vec<&str> = scramble.split_whitespace().collect();
            let tail = &moves[moves.len().saturating_sub(2)..];
            let reoriented = tail.iter().filter(|m| m.starts_with("3") && m.contains('w')).count();
            assert!(moves.len() <= 62, "{scramble}");
            // 4 of the 24 orientations need no move, the rest end in 3Rw/3Fw/3Uw.
            assert!(reoriented <= 2, "{scramble}");
        }
    }
}
