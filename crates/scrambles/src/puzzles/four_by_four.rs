//! 4x4x4 scramble generation using the threephase random-state solver.
//!
//! Port of Java's `FourByFourCubePuzzle` and `NoInspectionFourByFourCubePuzzle`.
//! The random-turn variant (`444fast`) lives in `big_cubes`.

use tnoodle_random::JavaRandom;
use tnoodle_threephase::search::Search;

use super::cube_puzzle::{CubePuzzle, CubeState};
use crate::algorithm_builder::{AlgorithmBuilder, MergingMode};
use crate::puzzle::{Puzzle, PuzzleStateAndGenerator, solve_in};

/// Random-state scramble, canonicalized like Java does with the threephase output.
fn generate_random_moves(rng: &mut JavaRandom) -> PuzzleStateAndGenerator<CubeState> {
    let scramble = Search::new().random_state(rng);
    let mut ab = AlgorithmBuilder::new(MergingMode::CanonicalizeMoves, CubeState::solved(4));
    ab.append_algorithm(&scramble)
        .expect("threephase emits valid 4x4 moves");
    ab.into_state_and_generator()
}

/// Reject scrambles that are too close to solved (Java `generateWcaScramble`).
fn wca_scramble(
    rng: &mut JavaRandom,
    mut generate: impl FnMut(&mut JavaRandom) -> PuzzleStateAndGenerator<CubeState>,
) -> String {
    let puzzle = CubePuzzle::new(4);
    loop {
        let psag = generate(rng);
        if solve_in(&puzzle, &psag.state, puzzle.wca_min_scramble_distance() - 1).is_none() {
            return psag.generator;
        }
    }
}

/// Generate a WCA-standard 4x4x4 scramble (random state).
pub fn generate_scramble(rng: &mut JavaRandom) -> String {
    wca_scramble(rng, generate_random_moves)
}

/// Generate a 4x4x4 blindfolded scramble: a random-state scramble followed by
/// a random whole-cube rotation such as `x y2`.
pub fn generate_scramble_ni(rng: &mut JavaRandom) -> String {
    let puzzle = CubePuzzle::new(4);
    let orientations = puzzle.random_orientation_moves(puzzle.size() - 1);
    wca_scramble(rng, |r| {
        let orientation = &orientations[r.next_index(orientations.len())];
        let psag = generate_random_moves(r);
        puzzle.apply_orientation(orientation, psag, false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::PuzzleState;

    #[test]
    fn generate_444_scramble() {
        let mut rng = JavaRandom::from_entropy();
        for _ in 0..5 {
            let scramble = generate_scramble(&mut rng);
            let moves = scramble.split_whitespace().count();
            // Java's threephase scrambles are typically 40-48 moves.
            assert!((35..=55).contains(&moves), "{moves} moves: {scramble}");
            assert!(
                !scramble.contains("Lw") && !scramble.contains("Dw") && !scramble.contains("Bw")
            );
            CubeState::solved(4).apply_algorithm(&scramble).unwrap();
        }
    }

    #[test]
    fn generate_444ni_scramble() {
        let mut rng = JavaRandom::from_entropy();
        for _ in 0..5 {
            let scramble = generate_scramble_ni(&mut rng);
            let moves: Vec<&str> = scramble.split_whitespace().collect();
            let rotations = moves
                .iter()
                .filter(|m| m.starts_with(['x', 'y', 'z']))
                .count();
            assert!(rotations <= 2, "{scramble}");
            assert!(
                moves[moves.len() - rotations..]
                    .iter()
                    .all(|m| m.starts_with(['x', 'y', 'z']))
            );
        }
    }
}
