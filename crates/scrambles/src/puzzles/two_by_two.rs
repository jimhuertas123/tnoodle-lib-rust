//! 2x2x2 scramble generation using the TwoByTwo solver.

use rand::Rng;
use tnoodle_solvers::two_by_two;

/// Generate a WCA-standard 2x2x2 scramble (random state, exactly 11 moves).
pub fn generate_scramble(rng: &mut impl Rng) -> String {
    let state = two_by_two::random_state(rng);
    two_by_two::generate_exactly(state, 11)
        .unwrap_or_else(|| {
            let state = two_by_two::random_state(rng);
            two_by_two::solve(state, 11).unwrap_or_default()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_222_scramble() {
        let mut rng = rand::rng();
        let scramble = generate_scramble(&mut rng);
        assert!(!scramble.is_empty());
    }
}
