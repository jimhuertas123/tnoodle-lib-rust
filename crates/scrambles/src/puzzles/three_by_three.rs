//! 3x3x3 scramble generation using the min2phase two-phase solver.

use rand::Rng;
use tnoodle_min2phase::{search::{Search, SearchResult}, tools};
use super::clean_scramble;

/// Generate a WCA-standard 3x3x3 scramble (random state, max 21 moves).
pub fn generate_scramble(rng: &mut impl Rng) -> String {
    let facelets = tools::random_state(rng);
    let mut search = Search::new();
    // INVERSE_SOLUTION (bit 1) = produce scramble, not solution
    let raw = match search.solution(&facelets, 21, 1_000_000, 0, 2) {
        SearchResult::Solution(s) => s,
        SearchResult::Error(_) => {
            let facelets = tools::random_state(rng);
            match search.solution(&facelets, 21, 10_000_000, 0, 2) {
                SearchResult::Solution(s) => s,
                SearchResult::Error(_) => return generate_random_moves_333(rng),
            }
        }
    };
    clean_scramble(&raw)
}

/// Generate a 3x3x3 scramble with no inspection (prepend random orientation).
pub fn generate_scramble_ni(rng: &mut impl Rng) -> String {
    let scramble = generate_scramble(rng);
    let orientations = ["", "z2", "z", "z'", "x", "x'"];
    let ori = orientations[rng.random_range(0..orientations.len())];
    if ori.is_empty() {
        scramble
    } else {
        format!("{ori} {scramble}")
    }
}

/// Generate a 3x3x3 FMC scramble (padded with R' U' F ... R' U' F).
pub fn generate_scramble_fm(rng: &mut impl Rng) -> String {
    let scramble = generate_scramble(rng);
    format!("R' U' F {scramble} R' U' F")
}

/// Fallback: random-turn 3x3x3 scramble (25 moves).
fn generate_random_moves_333(rng: &mut impl Rng) -> String {
    let faces = ["U", "R", "F", "D", "L", "B"];
    let dirs = ["", "2", "'"];
    let mut moves = Vec::with_capacity(25);
    let mut last_face = usize::MAX;

    for _ in 0..25 {
        loop {
            let face = rng.random_range(0..6);
            if face == last_face { continue; }
            let dir = rng.random_range(0..3);
            moves.push(format!("{}{}", faces[face], dirs[dir]));
            last_face = face;
            break;
        }
    }
    moves.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_333_scramble() {
        let mut rng = rand::rng();
        let scramble = generate_scramble(&mut rng);
        assert!(!scramble.is_empty(), "scramble should not be empty");
    }

    #[test]
    fn generate_333_no_consecutive_same_face() {
        let mut rng = rand::rng();
        for _ in 0..20 {
            let scramble = generate_scramble(&mut rng);
            let moves: Vec<&str> = scramble.split_whitespace().collect();
            for i in 1..moves.len() {
                let prev = moves[i - 1].chars().next().unwrap();
                let curr = moves[i].chars().next().unwrap();
                assert_ne!(prev, curr,
                    "consecutive same face '{prev}' in scramble: {scramble}");
            }
        }
    }

    #[test]
    fn clean_scramble_merges() {
        use super::super::clean_scramble;
        assert_eq!(clean_scramble("U' U'"), "U2");
        assert_eq!(clean_scramble("R R"), "R2");
        assert_eq!(clean_scramble("R R'"), "");
        assert_eq!(clean_scramble("R R R"), "R'");
        assert_eq!(clean_scramble("R R R R"), "");
        assert_eq!(clean_scramble("R2 R2"), "");
        assert_eq!(clean_scramble("U R U"), "U R U"); // different faces, no merge
    }

    #[test]
    fn generate_333_fm_scramble() {
        let mut rng = rand::rng();
        let scramble = generate_scramble_fm(&mut rng);
        assert!(scramble.starts_with("R' U' F "));
        assert!(scramble.ends_with(" R' U' F"));
    }
}
