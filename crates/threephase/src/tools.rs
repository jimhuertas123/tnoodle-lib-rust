#![allow(clippy::all)]
//! Port of cs.threephase.Tools
//!
//! Utility functions for the 4x4 three-phase solver:
//! - `random_cube`: generate a random 4x4 facelet string.

use rand::Rng;

use crate::full_cube::FullCube;

/// Generate a random 4x4x4 cube as a 96-character facelet string.
///
/// The string uses the standard URFDLB face ordering, with 16 sub-facelets
/// per face arranged in left-to-right, top-to-bottom order.
pub fn random_cube<R: Rng>(rng: &mut R) -> String {
    let mut c = FullCube::random(rng);
    let mut f = [0u8; 96];
    c.to_facelet(&mut f);
    let faces = b"URFDLB";
    f.iter().map(|&b| faces[b as usize] as char).collect()
}

// ─── tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_cube_length() {
        let mut rng = rand::rng();
        let s = random_cube(&mut rng);
        assert_eq!(s.len(), 96);
    }

    #[test]
    fn test_random_cube_valid_chars() {
        let mut rng = rand::rng();
        let s = random_cube(&mut rng);
        for ch in s.chars() {
            assert!(
                "URFDLB".contains(ch),
                "unexpected character '{}' in facelet string",
                ch
            );
        }
    }

    #[test]
    fn test_random_cube_face_counts() {
        let mut rng = rand::rng();
        let s = random_cube(&mut rng);
        // Each face character should appear exactly 16 times.
        for face in "URFDLB".chars() {
            let count = s.chars().filter(|&c| c == face).count();
            assert_eq!(
                count, 16,
                "face '{}' appears {} times (expected 16)",
                face, count
            );
        }
    }

    #[test]
    fn test_random_cube_deterministic_with_seed() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(42);
        let s1 = random_cube(&mut rng1);
        let s2 = random_cube(&mut rng2);
        assert_eq!(s1, s2, "same seed should produce same cube");
    }
}
