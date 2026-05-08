//! Pyraminx scramble generation using the Pyraminx solver.

use rand::Rng;
use tnoodle_solvers::pyraminx;

/// Generate a WCA-standard Pyraminx scramble (random state, exactly 11 moves + tips).
pub fn generate_scramble(rng: &mut impl Rng) -> String {
    let state = pyraminx::random_state(rng);
    pyraminx::generate_exactly(state, 11)
        .unwrap_or_else(|| {
            let state = pyraminx::random_state(rng);
            pyraminx::solve(state, 11).unwrap_or_default()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_pyra_scramble() {
        let mut rng = rand::rng();
        let scramble = generate_scramble(&mut rng);
        assert!(!scramble.is_empty());
    }
}
