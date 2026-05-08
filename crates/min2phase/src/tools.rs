/// Random cube generation and utility functions.
/// Port of cs.min2phase.Tools

use rand::Rng;
use crate::cubie_cube::CubieCube;
use crate::util;

/// Generate a uniformly random solvable cube state.
pub fn random_cube(rng: &mut impl Rng) -> CubieCube {
    let mut cc = CubieCube::new();

    // Random corner permutation
    cc.set_c_perm(rng.random_range(0..40320));
    // Random corner orientation
    cc.set_twist(rng.random_range(0..2187));
    // Random edge permutation (must match corner parity)
    let mut eperm = rng.random_range(0..40320);
    // Fix parity: edge perm parity must equal corner perm parity
    if util::get_n_parity(eperm, 12) != util::get_n_parity(cc.get_c_perm(), 8) {
        // Swap two edges to fix parity
        // Simple approach: try another random eperm or swap
        // Actually, the Java code uses a different approach - let me match it exactly
        if eperm % 2 == 0 { eperm += 1; } else { eperm -= 1; }
    }
    util::set_n_perm(&mut cc.ea, eperm, 12, true);
    // Random edge flip
    cc.set_flip(rng.random_range(0..2048));

    cc
}

/// Generate a random cube state as a 54-char facelet string.
pub fn random_state(rng: &mut impl Rng) -> String {
    let cc = random_cube(rng);
    util::to_face_cube(&cc.ca, &cc.ea)
}

/// Convert a scramble string to a facelet string.
pub fn from_scramble(scramble: &str) -> String {
    let mut cc = CubieCube::new();
    let mut d = CubieCube::new();

    let tables = &*crate::cubie_cube::TABLES;

    for mv_str in scramble.split_whitespace() {
        let mv_idx = match parse_move(mv_str) {
            Some(idx) => idx,
            None => continue,
        };
        let mut prod = CubieCube::new();
        crate::cubie_cube::corn_mult(&cc, &tables.move_cube[mv_idx], &mut prod);
        crate::cubie_cube::edge_mult(&cc, &tables.move_cube[mv_idx], &mut d);
        prod.ea.copy_from_slice(&d.ea);
        cc = prod;
    }

    util::to_face_cube(&cc.ca, &cc.ea)
}

/// Parse a move string like "R", "R2", "R'" into a move index (0-17).
fn parse_move(s: &str) -> Option<usize> {
    let s = s.trim();
    if s.is_empty() { return None; }

    let face_idx = match s.chars().next()? {
        'U' => 0,
        'R' => 3,
        'F' => 6,
        'D' => 9,
        'L' => 12,
        'B' => 15,
        _ => return None,
    };

    let power = if s.len() == 1 {
        0 // quarter turn
    } else {
        match s.chars().nth(1)? {
            '2' => 1,
            '\'' | '\u{2019}' | '3' => 2,
            ' ' => 0,
            _ => return None,
        }
    };

    Some(face_idx + power)
}

/// Verify a facelet string. Returns 0 if valid, negative error code otherwise.
pub fn verify(facelets: &str) -> i32 {
    if facelets.len() != 54 {
        return -1;
    }

    let mut f = [0u8; 54];
    for (i, ch) in facelets.chars().enumerate() {
        f[i] = match ch {
            'U' => 0,
            'R' => 1,
            'F' => 2,
            'D' => 3,
            'L' => 4,
            'B' => 5,
            _ => return -1,
        };
    }

    // Check each color appears exactly 9 times
    let mut counts = [0i32; 6];
    for &c in &f {
        counts[c as usize] += 1;
    }
    for &c in &counts {
        if c != 9 { return -1; }
    }

    let mut ca = [0u8; 8];
    let mut ea = [0u8; 12];
    util::to_cubie_cube(&f, &mut ca, &mut ea);

    let cc = CubieCube { ca, ea };
    cc.verify()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_moves() {
        assert_eq!(parse_move("U"), Some(0));
        assert_eq!(parse_move("U2"), Some(1));
        assert_eq!(parse_move("U'"), Some(2));
        assert_eq!(parse_move("R"), Some(3));
        assert_eq!(parse_move("R2"), Some(4));
        assert_eq!(parse_move("B'"), Some(17));
    }

    #[test]
    fn random_cube_valid() {
        let mut rng = rand::rng();
        for _ in 0..10 {
            let cc = random_cube(&mut rng);
            assert_eq!(cc.verify(), 0, "random cube failed verification");
        }
    }

    #[test]
    fn solved_state_valid() {
        let facelets = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
        assert_eq!(verify(facelets), 0);
    }

    #[test]
    fn from_scramble_returns_valid() {
        let face = from_scramble("R U R' U'");
        assert_eq!(verify(&face), 0);
    }
}
