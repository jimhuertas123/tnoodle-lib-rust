//! Pyraminx scramble generation using the Pyraminx solver.

use rand::Rng;
use tnoodle_solvers::pyraminx;

/// Number of non-tip moves (Java's `PyraminxPuzzle.MIN_SCRAMBLE_LENGTH`).
const SCRAMBLE_LENGTH: usize = 11;
/// Java's `PyraminxPuzzle.wcaMinScrambleDistance`, counting tip turns.
const MIN_SCRAMBLE_DISTANCE: usize = 6;

/// Generate a WCA-standard Pyraminx scramble: random state, exactly 11 moves
/// followed by 0-4 tip turns, at least 6 moves (tips included) from solved.
pub fn generate_scramble(rng: &mut impl Rng) -> String {
    loop {
        let state = pyraminx::random_state(rng);
        let tips = state.unsolved_tips();
        // `solve` does not count tips, so subtract them from the budget.
        let too_close = MIN_SCRAMBLE_DISTANCE - 1 >= tips
            && pyraminx::solve(state, MIN_SCRAMBLE_DISTANCE - 1 - tips).is_some();
        if too_close {
            continue;
        }
        // `generate_exactly` counts tips in its length.
        if let Some(scramble) = pyraminx::generate_exactly(state, SCRAMBLE_LENGTH + tips) {
            return scramble;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_pyra_scramble() {
        let mut rng = rand::rng();
        for _ in 0..50 {
            let scramble = generate_scramble(&mut rng);
            let moves: Vec<&str> = scramble.split_whitespace().collect();
            let tips = moves.iter().filter(|m| m.starts_with(char::is_lowercase)).count();
            assert_eq!(moves.len() - tips, SCRAMBLE_LENGTH, "{scramble}");
            assert!(tips <= 4, "{scramble}");
            // Tips come last.
            assert!(moves[SCRAMBLE_LENGTH..].iter().all(|m| m.starts_with(char::is_lowercase)), "{scramble}");
        }
    }
}
