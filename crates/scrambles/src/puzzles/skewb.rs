//! Skewb scramble generation using the Skewb solver.

use rand::Rng;
use tnoodle_solvers::skewb;

/// Generate a WCA-standard Skewb scramble (random state, exactly 11 moves).
pub fn generate_scramble(rng: &mut impl Rng) -> String {
    let state = skewb::random_state(rng);
    skewb::generate_exactly(state, 11)
        .unwrap_or_else(|| {
            let state = skewb::random_state(rng);
            skewb::solve(state, 11).unwrap_or_default()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_skewb_scramble() {
        let mut rng = rand::rng();
        let scramble = generate_scramble(&mut rng);
        assert!(!scramble.is_empty());
    }
}
