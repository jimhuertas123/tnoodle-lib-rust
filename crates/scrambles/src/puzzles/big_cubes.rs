//! Random-turn scramble generation for 4x4x4 (fast), 5x5x5, 6x6x6, 7x7x7 cubes.

use rand::Rng;

/// Default scramble lengths per cube size (index = size).
const DEFAULT_LENGTHS: [usize; 12] = [0, 0, 25, 25, 40, 60, 80, 100, 120, 140, 160, 180];

/// Generate a random-turn scramble for an NxN cube (N >= 4).
pub fn generate_scramble(size: usize, rng: &mut (impl Rng + ?Sized)) -> String {
    let count = if size < DEFAULT_LENGTHS.len() {
        DEFAULT_LENGTHS[size]
    } else {
        size * 20
    };

    let faces = ["U", "R", "F", "D", "L", "B"];
    let dirs = ["", "2", "'"];
    let max_slice = size / 2;

    let mut moves = Vec::with_capacity(count);
    // Track the face AXIS (0=U/D, 1=R/L, 2=F/B) of recent moves
    let mut last_face: usize = usize::MAX;       // face index (0-5) of last move
    let mut second_last_face: usize = usize::MAX; // face index of move before last

    for _ in 0..count {
        loop {
            let face = rng.random_range(0..6);
            let face_axis = face / 2; // U/D=0, R/L=1, F/B=2

            // Rule 1: No two consecutive moves on the same face
            // (e.g., R then Rw is redundant — they're both on the R face)
            if last_face != usize::MAX && face == last_face {
                continue;
            }

            // Rule 2: No three consecutive moves on the same axis
            // (e.g., R L R is redundant — R and L are on the same axis)
            if last_face != usize::MAX && second_last_face != usize::MAX {
                let last_axis = last_face / 2;
                let second_last_axis = second_last_face / 2;
                if face_axis == last_axis && face_axis == second_last_axis {
                    continue;
                }
            }

            // Rule 3: On same axis, enforce ordering (e.g., allow R then L but not L then R)
            // This prevents equivalent sequences like "R L" and "L R"
            if last_face != usize::MAX {
                let last_axis = last_face / 2;
                if face_axis == last_axis && face > last_face {
                    continue;
                }
            }

            let dir = rng.random_range(0..3);
            let slice = rng.random_range(0..max_slice);

            let mv = if slice == 0 {
                format!("{}{}", faces[face], dirs[dir])
            } else if slice == 1 {
                format!("{}w{}", faces[face], dirs[dir])
            } else {
                format!("{}{}w{}", slice + 1, faces[face], dirs[dir])
            };

            moves.push(mv);
            second_last_face = last_face;
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
    fn generate_555_scramble() {
        let mut rng = rand::rng();
        let scramble = generate_scramble(5, &mut rng);
        let count = scramble.split_whitespace().count();
        assert_eq!(count, 60);
    }

    #[test]
    fn generate_777_scramble() {
        let mut rng = rand::rng();
        let scramble = generate_scramble(7, &mut rng);
        let count = scramble.split_whitespace().count();
        assert_eq!(count, 100);
    }

    #[test]
    fn no_consecutive_same_face() {
        let mut rng = rand::rng();
        for _ in 0..20 {
            let scramble = generate_scramble(5, &mut rng);
            let moves: Vec<&str> = scramble.split_whitespace().collect();
            for i in 1..moves.len() {
                let prev_face = moves[i - 1].chars().next().unwrap();
                let curr_face = moves[i].chars().next().unwrap();
                assert_ne!(prev_face, curr_face,
                    "consecutive same face in: {scramble}");
            }
        }
    }
}
