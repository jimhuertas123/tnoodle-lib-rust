//! Square-1 scramble generation using the sq12phase solver.

use rand::Rng;
use tnoodle_sq12phase::{full_cube::FullCube, search::{INVERSE_SOLUTION, Search}};

/// Java's `SquareOnePuzzle.wcaMinScrambleDistance`, in the WCA turn metric.
const MIN_SCRAMBLE_DISTANCE: i32 = 11;

/// Generate a WCA-standard Square-1 scramble (random state, at least 11
/// moves away from solved).
pub fn generate_scramble(rng: &mut impl Rng) -> String {
    let mut search = Search::new();
    loop {
        let cube = FullCube::random_cube(rng);
        if search.solution_opt(&cube, MIN_SCRAMBLE_DISTANCE - 1).is_some() {
            continue;
        }
        if let Some(scramble) = search.solution_with_verbose(&cube, INVERSE_SOLUTION) {
            return scramble.trim().to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_sq1_scramble() {
        let mut rng = rand::rng();
        let scramble = generate_scramble(&mut rng);
        assert!(scramble.contains('/'), "{scramble}");
    }
}
