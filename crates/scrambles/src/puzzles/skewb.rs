//! Skewb scramble generation using the Skewb solver.

use tnoodle_random::JavaRandom;
use tnoodle_solvers::skewb;

const SCRAMBLE_LENGTH: usize = 11;
/// Java's `SkewbPuzzle.wcaMinScrambleDistance`.
const MIN_SCRAMBLE_DISTANCE: usize = 7;

/// Generate a WCA-standard Skewb scramble (random state, exactly 11 moves,
/// at least 7 moves away from solved).
pub fn generate_scramble(rng: &mut JavaRandom) -> String {
    loop {
        let state = skewb::random_state(rng);
        if skewb::solve(state, MIN_SCRAMBLE_DISTANCE - 1).is_some() {
            continue;
        }
        if let Some(scramble) = skewb::generate_exactly(state, SCRAMBLE_LENGTH) {
            return scramble;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_skewb_scramble() {
        let mut rng = JavaRandom::from_entropy();
        for _ in 0..50 {
            let scramble = generate_scramble(&mut rng);
            let moves: Vec<&str> = scramble.split_whitespace().collect();
            assert_eq!(moves.len(), SCRAMBLE_LENGTH, "{scramble}");
            for w in moves.windows(2) {
                assert_ne!(&w[0][..1], &w[1][..1], "same corner twice: {scramble}");
            }
        }
    }
}
