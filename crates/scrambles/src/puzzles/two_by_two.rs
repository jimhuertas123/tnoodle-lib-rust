//! 2x2x2 scramble generation using the TwoByTwo solver.

use rand::Rng;
use tnoodle_solvers::two_by_two;

const SCRAMBLE_LENGTH: usize = 11;
/// Java's `TwoByTwoCubePuzzle.wcaMinScrambleDistance`.
const MIN_SCRAMBLE_DISTANCE: usize = 4;

/// Generate a WCA-standard 2x2x2 scramble (random state, exactly 11 moves,
/// at least 4 moves away from solved).
pub fn generate_scramble(rng: &mut impl Rng) -> String {
    loop {
        let state = two_by_two::random_state(rng);
        if two_by_two::solve(state, MIN_SCRAMBLE_DISTANCE - 1).is_some() {
            continue;
        }
        if let Some(scramble) = two_by_two::generate_exactly(state, SCRAMBLE_LENGTH) {
            return scramble;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::PuzzleState;
    use crate::puzzles::cube_puzzle::CubeState;

    #[test]
    fn generate_222_scramble() {
        let mut rng = rand::rng();
        for _ in 0..50 {
            let scramble = generate_scramble(&mut rng);
            assert_eq!(scramble.split_whitespace().count(), SCRAMBLE_LENGTH, "{scramble}");
            CubeState::solved(2).apply_algorithm(&scramble).expect("valid 2x2 moves");
        }
    }

    #[test]
    fn solver_solution_undoes_generated_scramble() {
        // Port of HugeScrambleTest.testTwosSolver: apply the scramble, then the
        // solver's solution for the same state, and check the cube is solved.
        let mut rng = rand::rng();
        let solved = CubeState::solved(2);
        for _ in 0..50 {
            let state = two_by_two::random_state(&mut rng);
            let scramble = two_by_two::generate_exactly(state, SCRAMBLE_LENGTH).unwrap();
            let solution = two_by_two::solve(state, SCRAMBLE_LENGTH).unwrap();
            let result = solved.apply_algorithm(&format!("{scramble} {solution}")).unwrap();
            assert!(result.equals_normalized(&solved), "{scramble} / {solution}");
        }
    }
}
