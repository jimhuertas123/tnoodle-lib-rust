//! Square-1 scramble generation using the sq12phase solver.

use rand::Rng;
use tnoodle_sq12phase::{full_cube::FullCube, search::Search};

/// Generate a WCA-standard Square-1 scramble.
pub fn generate_scramble(rng: &mut impl Rng) -> String {
    let cube = FullCube::random_cube(rng);
    let mut search = Search::new();
    search.solution(&cube)
        .unwrap_or_else(|| {
            let cube = FullCube::random_cube(rng);
            search.solution(&cube).unwrap_or_default()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_sq1_scramble() {
        let mut rng = rand::rng();
        let scramble = generate_scramble(&mut rng);
        assert!(!scramble.is_empty());
    }
}
