//! Square-1 scramble generation using the sq12phase solver.

use rand::Rng;
use tnoodle_sq12phase::{full_cube::FullCube, search::{INVERSE_SOLUTION, Search}};

/// Generate a WCA-standard Square-1 scramble (random state).
///
/// Java also rejects states within 10 moves of solved using an optimal
/// solver (`solutionOpt`), which is not ported; random states that close are
/// vanishingly rare.
pub fn generate_scramble(rng: &mut impl Rng) -> String {
    let mut search = Search::new();
    loop {
        let cube = FullCube::random_cube(rng);
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
