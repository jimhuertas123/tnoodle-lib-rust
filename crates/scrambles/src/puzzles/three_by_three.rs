//! 3x3x3 scramble generation using the min2phase two-phase solver.
//!
//! Port of Java's `ThreeByThreeCubePuzzle`, `NoInspectionThreeByThreeCubePuzzle`
//! and `ThreeByThreeCubeFewestMovesPuzzle`.

use rand::Rng;
use tnoodle_min2phase::{search::{Search, SearchResult}, tools};

use super::cube_puzzle::{CubePuzzle, CubeState};
use crate::algorithm_builder::{AlgorithmBuilder, MergingMode};
use crate::puzzle::{Puzzle, PuzzleStateAndGenerator, solve_in};

const MAX_SCRAMBLE_LENGTH: i32 = 21;
const MAX_PROBE: i64 = 1_000_000;
/// min2phase verbose flag: return the inverse of the solution (the scramble).
const INVERSE_SOLUTION: u32 = 2;
const FM_PADDING: &str = "R' U' F";

/// Random-state scramble whose underlying solution may not start on
/// `first_axis` nor end on `last_axis` (so the scramble may not end on
/// `first_axis` nor start on `last_axis`).
fn generate_random_moves(
    rng: &mut impl Rng,
    first_axis: Option<&str>,
    last_axis: Option<&str>,
) -> PuzzleStateAndGenerator<CubeState> {
    let mut search = Search::new();
    let scramble = loop {
        let facelets = tools::random_state(rng);
        let result = search.solution_with_restrictions(
            &facelets,
            MAX_SCRAMBLE_LENGTH,
            MAX_PROBE,
            0,
            INVERSE_SOLUTION,
            first_axis,
            last_axis,
        );
        if let SearchResult::Solution(scramble) = result {
            break scramble;
        }
    };
    // Same as Java: canonicalize the solver output so it can never contain
    // redundant moves.
    let mut ab = AlgorithmBuilder::new(MergingMode::CanonicalizeMoves, CubeState::solved(3));
    ab.append_algorithm(&scramble).expect("min2phase emits valid moves");
    ab.into_state_and_generator()
}

/// Reject scrambles that are too close to solved (Java `generateWcaScramble`).
fn wca_scramble<R: Rng>(
    rng: &mut R,
    mut generate: impl FnMut(&mut R) -> PuzzleStateAndGenerator<CubeState>,
) -> String {
    let puzzle = CubePuzzle::new(3);
    loop {
        let psag = generate(rng);
        if solve_in(&puzzle, &psag.state, puzzle.wca_min_scramble_distance() - 1).is_none() {
            return psag.generator;
        }
    }
}

/// Generate a WCA-standard 3x3x3 scramble (random state, max 21 moves).
pub fn generate_scramble(rng: &mut impl Rng) -> String {
    wca_scramble(rng, |r| generate_random_moves(r, None, None))
}

/// Generate a 3x3x3 blindfolded scramble: random state followed by a random
/// reorientation such as `Rw Uw'`.
pub fn generate_scramble_ni(rng: &mut impl Rng) -> String {
    let puzzle = CubePuzzle::new(3);
    let orientations = puzzle.random_orientation_moves(puzzle.size() / 2);
    wca_scramble(rng, |r| {
        let orientation = &orientations[r.random_range(0..orientations.len())];
        // The scramble must not end on the axis the reorientation starts with.
        let first_axis = orientation.first().map(|m| &m[..1]);
        let psag = generate_random_moves(r, first_axis, None);
        puzzle.apply_orientation(orientation, psag, false)
    })
}

/// Generate a 3x3x3 Fewest Moves scramble, padded with `R' U' F` on both ends.
pub fn generate_scramble_fm(rng: &mut impl Rng) -> String {
    wca_scramble(rng, |r| {
        // The solution may not start on the suffix's first axis (R) nor end on
        // the prefix's last axis (F), so nothing cancels with the padding.
        let psag = generate_random_moves(r, Some("R"), Some("F"));
        let mut ab = AlgorithmBuilder::new(MergingMode::NoMerging, CubeState::solved(3));
        for part in [FM_PADDING, psag.generator.as_str(), FM_PADDING] {
            ab.append_algorithm(part).expect("valid moves");
        }
        ab.into_state_and_generator()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::PuzzleState;

    #[test]
    fn generate_333_fm_scramble() {
        // Port of ThreeByThreeCubeFewestMovesTest: the padding never cancels,
        // so canonicalizing the scramble leaves it unchanged.
        let mut rng = rand::rng();
        for _ in 0..20 {
            let scramble = generate_scramble_fm(&mut rng);
            assert!(scramble.starts_with("R' U' F "), "{scramble}");
            assert!(scramble.ends_with(" R' U' F"), "{scramble}");
            let mut ab = AlgorithmBuilder::new(MergingMode::CanonicalizeMoves, CubeState::solved(3));
            ab.append_algorithm(&scramble).unwrap();
            assert_eq!(ab.to_string(), scramble);
        }
    }

    #[test]
    fn generate_333ni_scramble() {
        let mut rng = rand::rng();
        let mut saw_orientation = false;
        for _ in 0..30 {
            let scramble = generate_scramble_ni(&mut rng);
            let moves: Vec<&str> = scramble.split_whitespace().collect();
            let wide: Vec<&&str> = moves.iter().filter(|m| m.contains('w')).collect();
            assert!(wide.len() <= 2, "{scramble}");
            // Wide moves only appear as the trailing reorientation.
            assert!(moves[moves.len() - wide.len()..].iter().all(|m| m.contains('w')), "{scramble}");
            saw_orientation |= !wide.is_empty();
            let state = CubeState::solved(3).apply_algorithm(&scramble).unwrap();
            assert!(!state.equals_normalized(&CubeState::solved(3)));
        }
        assert!(saw_orientation);
    }
}
