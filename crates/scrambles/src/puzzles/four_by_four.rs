//! 4x4x4 scramble generation.
//!
//! Uses random-turn scrambles (40 moves) matching WCA standards.

use rand::Rng;
use super::big_cubes;

/// Generate a WCA-standard 4x4x4 scramble (random turns, 40 moves).
pub fn generate_scramble(rng: &mut impl Rng) -> String {
    big_cubes::generate_scramble(4, rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_444_scramble() {
        let mut rng = rand::rng();
        let scramble = generate_scramble(&mut rng);
        assert!(!scramble.is_empty());
        assert_eq!(scramble.split_whitespace().count(), 40);
    }

    #[test]
    fn no_consecutive_same_face() {
        let mut rng = rand::rng();
        let scramble = generate_scramble(&mut rng);
        let moves: Vec<&str> = scramble.split_whitespace().collect();
        for i in 1..moves.len() {
            let prev_face = moves[i - 1].chars().next().unwrap();
            let curr_face = moves[i].chars().next().unwrap();
            assert_ne!(prev_face, curr_face,
                "consecutive same face in: {scramble}");
        }
    }
}
