//! NxN cube puzzle model used for random-turn scrambles, move canonicalization
//! and scramble validation.
//!
//! Port of Java's `CubePuzzle` / `CubePuzzle.CubeState`. Stickers are stored in
//! a flat `[face][row][col]` image (faces ordered R, U, F, L, D, B like Java's
//! `Face` enum). Every named move and every whole-cube rotation is precomputed
//! per cube size as a sticker permutation, so applying a move is one gather.

use std::collections::HashMap;
use std::sync::OnceLock;

use rand::Rng;
use tnoodle_svglite::{Color, Dimension, Svg};

use crate::algorithm_builder::{AlgorithmBuilder, MergingMode};
use crate::puzzle::{Puzzle, PuzzleError, PuzzleState, PuzzleStateAndGenerator};

use super::cube_svg::{self, NUM_FACES};

/// Face names in Java `Face` enum order.
const FACE_NAMES: [&str; NUM_FACES] = ["R", "U", "F", "L", "D", "B"];
const R: u8 = 0;
const U: u8 = 1;
const F: u8 = 2;
const L: u8 = 3;
const D: u8 = 4;
const B: u8 = 5;

const DIR_TO_STR: [&str; 4] = ["", "", "2", "'"];
const DEFAULT_LENGTHS: [i32; 12] = [0, 0, 25, 25, 40, 60, 80, 100, 120, 140, 160, 180];
const MAX_SIZE: usize = 11;

/// Name of a move turning slices `0..=inner_slice` of `face`, or `None` for
/// unnamed rotations. Port of Java's `CubeMove.toString`.
pub fn move_name(size: usize, face: u8, dir: usize, inner_slice: usize) -> Option<String> {
    let f = FACE_NAMES[face as usize];
    let base = if inner_slice == 0 {
        f.to_string()
    } else if inner_slice == 1 {
        format!("{f}w")
    } else if inner_slice == size - 1 {
        // Turning all the slices is a rotation; only R, U, F rotations are named.
        match face {
            R => "x".to_string(),
            U => "y".to_string(),
            F => "z".to_string(),
            _ => return None,
        }
    } else {
        format!("{}{f}w", inner_slice + 1)
    };
    Some(base + DIR_TO_STR[dir])
}

// ---------------------------------------------------------------------------
// Per-size tables
// ---------------------------------------------------------------------------

/// `perm[i]` = index of the sticker that moves into position `i`.
type Perm = Vec<u16>;

struct CubeTables {
    /// All named moves (Java `getSuccessorsByName`), in Java iteration order.
    moves: Vec<(String, Perm)>,
    by_name: HashMap<String, usize>,
    /// Indices into `moves` of the scramble moves (Java `getScrambleSuccessors`).
    scramble_moves: Vec<usize>,
    /// The 24 whole-cube rotations.
    rotations: Vec<Perm>,
}

static TABLES: [OnceLock<CubeTables>; MAX_SIZE + 1] = [const { OnceLock::new() }; MAX_SIZE + 1];

fn tables(size: usize) -> &'static CubeTables {
    TABLES[size].get_or_init(|| CubeTables::new(size))
}

fn sticker_index(size: usize, face: usize, row: usize, col: usize) -> usize {
    (face * size + row) * size + col
}

/// Sticker permutation of turning slices `0..=inner_slice` of `face` by `dir`.
fn slice_perm(size: usize, face: u8, dir: usize, inner_slice: usize) -> Perm {
    let mut image: Vec<Vec<Vec<u16>>> = (0..NUM_FACES)
        .map(|f| {
            (0..size)
                .map(|r| (0..size).map(|c| sticker_index(size, f, r, c) as u16).collect())
                .collect()
        })
        .collect();
    for slice in 0..=inner_slice {
        cube_svg::apply_slice(&mut image, size, face, slice, dir);
    }
    image.into_iter().flatten().flatten().collect()
}

fn compose(first: &Perm, then: &Perm) -> Perm {
    then.iter().map(|&i| first[i as usize]).collect()
}

impl CubeTables {
    fn new(size: usize) -> Self {
        let mut moves = Vec::new();
        let mut scramble_moves = Vec::new();
        for inner_slice in 0..size {
            for face in 0..NUM_FACES as u8 {
                let half_of_even_cube = size % 2 == 0 && inner_slice == size / 2 - 1;
                // Skip turning the other halves of even sized cubes.
                let scramble_move =
                    inner_slice < size / 2 && !(face >= 3 && half_of_even_cube);
                for dir in 1..=3 {
                    let Some(name) = move_name(size, face, dir, inner_slice) else {
                        continue;
                    };
                    if scramble_move {
                        scramble_moves.push(moves.len());
                    }
                    moves.push((name, slice_perm(size, face, dir, inner_slice)));
                }
            }
        }
        let by_name = moves
            .iter()
            .enumerate()
            .map(|(i, (name, _))| (name.clone(), i))
            .collect();

        // Close {x, y, z} under composition to get all 24 rotations.
        let generators: Vec<Perm> = [R, U, F]
            .iter()
            .map(|&face| slice_perm(size, face, 1, size - 1))
            .collect();
        let identity: Perm = (0..(NUM_FACES * size * size) as u16).collect();
        let mut rotations = vec![identity];
        let mut i = 0;
        while i < rotations.len() {
            for g in &generators {
                let next = compose(&rotations[i], g);
                if !rotations.contains(&next) {
                    rotations.push(next);
                }
            }
            i += 1;
        }
        debug_assert_eq!(rotations.len(), 24);

        Self { moves, by_name, scramble_moves, rotations }
    }
}

// ---------------------------------------------------------------------------
// CubeState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CubeState {
    size: usize,
    image: Vec<u8>,
}

impl CubeState {
    pub fn solved(size: usize) -> Self {
        let image = (0..NUM_FACES)
            .flat_map(|f| std::iter::repeat_n(f as u8, size * size))
            .collect();
        Self { size, image }
    }

    fn permuted(&self, perm: &Perm) -> Self {
        Self {
            size: self.size,
            image: perm.iter().map(|&i| self.image[i as usize]).collect(),
        }
    }

    fn sticker(&self, face: u8, row: usize, col: usize) -> u8 {
        self.image[sticker_index(self.size, face as usize, row, col)]
    }

    /// Whether the BLD corner is solved, which pins down the orientation.
    fn bld_corner_solved(&self) -> bool {
        let s = self.size - 1;
        self.sticker(B, s, s) == B && self.sticker(L, s, 0) == L && self.sticker(D, s, 0) == D
    }

    /// Sticker colors as face letters, faces in URFDLB order, each face in
    /// reading order. Port of Java's `CubeState.toFaceCube` (which only
    /// supports 3x3x3), generalized to any size.
    pub fn to_face_cube(&self) -> String {
        let n = self.size;
        [U, R, F, D, L, B]
            .iter()
            .flat_map(|&face| (0..n).flat_map(move |r| (0..n).map(move |c| (face, r, c))))
            .map(|(face, r, c)| FACE_NAMES[self.sticker(face, r, c) as usize])
            .collect()
    }

    fn successor(&self, move_idx: usize) -> Self {
        self.permuted(&tables(self.size).moves[move_idx].1)
    }
}

impl PuzzleState for CubeState {
    fn successors_by_name(&self) -> HashMap<String, Self> {
        tables(self.size)
            .moves
            .iter()
            .map(|(name, perm)| (name.clone(), self.permuted(perm)))
            .collect()
    }

    fn scramble_successors(&self) -> HashMap<String, Self> {
        let t = tables(self.size);
        t.scramble_moves
            .iter()
            .map(|&i| (t.moves[i].0.clone(), self.successor(i)))
            .collect()
    }

    /// Java's `CubeState.getCanonicalMovesByState` is the scramble successors.
    fn canonical_moves_by_state(&self) -> HashMap<Self, String> {
        let t = tables(self.size);
        t.scramble_moves
            .iter()
            .map(|&i| (self.successor(i), t.moves[i].0.clone()))
            .collect()
    }

    fn get_normalized(&self) -> Self {
        // A state is normalized when the BLD corner is solved. Exactly one of
        // the 24 rotations achieves that.
        tables(self.size)
            .rotations
            .iter()
            .map(|rot| self.permuted(rot))
            .find(CubeState::bld_corner_solved)
            .expect("one rotation must solve the BLD corner")
    }

    fn is_normalized(&self) -> bool {
        self.bld_corner_solved()
    }

    fn draw_scramble(&self, _colors: &HashMap<String, Color>) -> Svg {
        let n = self.size;
        let image: Vec<Vec<Vec<u8>>> = self
            .image
            .chunks(n * n)
            .map(|face| face.chunks(n).map(<[u8]>::to_vec).collect())
            .collect();
        cube_svg::render_image(n, &image)
    }

    fn apply(&self, mv: &str) -> Result<Self, PuzzleError> {
        tables(self.size)
            .by_name
            .get(mv)
            .map(|&i| self.successor(i))
            .ok_or_else(|| PuzzleError::InvalidMove(format!("Unrecognized turn {mv}")))
    }
}

// ---------------------------------------------------------------------------
// CubePuzzle
// ---------------------------------------------------------------------------

pub struct CubePuzzle {
    size: usize,
    short_name: String,
    long_name: String,
}

impl CubePuzzle {
    pub fn new(size: usize) -> Self {
        assert!((2..=MAX_SIZE).contains(&size), "Invalid cube size");
        Self {
            size,
            short_name: format!("{size}{size}{size}"),
            long_name: format!("{size}x{size}x{size}"),
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    /// The 24 ways to reorient the cube, as move names turning `thickness + 1`
    /// layers. Port of Java's `getRandomOrientationMoves`.
    pub fn random_orientation_moves(&self, thickness: usize) -> Vec<Vec<String>> {
        let name = |face, dir| move_name(self.size, face, dir, thickness).expect("named move");
        let u_face_moves = [None, Some((R, 1)), Some((R, 2)), Some((R, 3)), Some((F, 1)), Some((F, 3))];
        let f_face_moves = [None, Some((U, 1)), Some((U, 2)), Some((U, 3))];
        let mut orientations = Vec::with_capacity(24);
        for u in u_face_moves {
            for f in f_face_moves {
                orientations.push(
                    [u, f].into_iter().flatten().map(|(face, dir)| name(face, dir)).collect(),
                );
            }
        }
        orientations
    }

    /// Port of Java's `applyOrientation` (NoInspection puzzles). With
    /// `discard_redundant_moves`, scramble moves that the reorientation would
    /// cancel or merge with are dropped first.
    pub fn apply_orientation(
        &self,
        orientation: &[String],
        psag: PuzzleStateAndGenerator<CubeState>,
        discard_redundant_moves: bool,
    ) -> PuzzleStateAndGenerator<CubeState> {
        let Some(first) = orientation.first() else {
            return psag;
        };
        let mut ab = AlgorithmBuilder::new(MergingMode::NoMerging, self.solved_state());
        ab.append_algorithm(&psag.generator).expect("generated scramble is valid");
        if discard_redundant_moves {
            while ab.is_redundant(first) {
                let index = ab
                    .find_best_index_for_move(first, MergingMode::CanonicalizeMoves)
                    .expect("orientation move is valid")
                    .index;
                ab.pop_move(index);
            }
        }
        for mv in orientation {
            ab.append_move(mv).expect("orientation move is valid");
        }
        ab.into_state_and_generator()
    }

    /// Random-turn scramble with a random reorientation appended.
    pub fn generate_with_orientation(
        &self,
        rng: &mut (impl Rng + ?Sized),
        thickness: usize,
        discard_redundant_moves: bool,
    ) -> PuzzleStateAndGenerator<CubeState> {
        let orientations = self.random_orientation_moves(thickness);
        let orientation = &orientations[rng.random_range(0..orientations.len())];
        let psag = self.generate_random_moves(rng);
        self.apply_orientation(orientation, psag, discard_redundant_moves)
    }
}

impl Puzzle for CubePuzzle {
    type State = CubeState;

    fn short_name(&self) -> &str {
        &self.short_name
    }

    fn long_name(&self) -> &str {
        &self.long_name
    }

    fn default_color_scheme(&self) -> HashMap<String, Color> {
        HashMap::new()
    }

    fn preferred_size(&self) -> Dimension {
        cube_svg::preferred_size(self.size)
    }

    fn solved_state(&self) -> CubeState {
        CubeState::solved(self.size)
    }

    fn random_move_count(&self) -> i32 {
        DEFAULT_LENGTHS[self.size]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_names_match_java() {
        assert_eq!(move_name(3, R, 1, 0).unwrap(), "R");
        assert_eq!(move_name(3, R, 3, 1).unwrap(), "Rw'");
        assert_eq!(move_name(3, R, 2, 2).unwrap(), "x2");
        assert_eq!(move_name(3, L, 1, 2), None);
        assert_eq!(move_name(5, U, 1, 3).unwrap(), "4Uw");
        assert_eq!(move_name(7, F, 3, 2).unwrap(), "3Fw'");
    }

    #[test]
    fn scramble_moves_skip_redundant_halves_on_even_cubes() {
        // Port of AlgorithmBuilderTest.testRedundantMoves (first half).
        let sixes = CubeState::solved(6).scramble_successors();
        for mv in ["3Bw", "3Lw", "3Dw", "x"] {
            assert!(sixes.keys().all(|m| !m.starts_with(mv)), "{mv} offered on 6x6");
        }
        assert!(sixes.contains_key("3Rw2"));
        let fours = CubeState::solved(4).scramble_successors();
        for mv in ["Lw", "Dw", "Bw"] {
            assert!(fours.keys().all(|m| !m.starts_with(mv)), "{mv} offered on 4x4");
        }
        assert_eq!(fours.len(), 6 * 3 + 3 * 3);
    }

    #[test]
    fn canonical_moves_match_java() {
        // From NoInspectionThreeByThreeTest / ThreeByThreeCubeFewestMovesTest.
        let names = |size| {
            let mut v: Vec<String> =
                CubeState::solved(size).canonical_moves_by_state().into_values().collect();
            v.sort();
            v
        };
        let expected = |faces: &str| {
            let mut v: Vec<String> = faces
                .chars()
                .flat_map(|f| ["", "'", "2"].map(|d| format!("{f}{d}")))
                .collect();
            v.sort();
            v
        };
        assert_eq!(names(2), expected("RUF"));
        assert_eq!(names(3), expected("RUFLDB"));
    }

    #[test]
    fn moves_have_order_four() {
        for size in 2..=7 {
            let solved = CubeState::solved(size);
            for (name, _) in &tables(size).moves {
                let state = solved.apply_algorithm(&format!("{name} {name} {name} {name}")).unwrap();
                assert_eq!(state, solved, "{name} on {size}x{size}");
            }
        }
    }

    #[test]
    fn normalization() {
        // Port of HugeScrambleTest.testCubeNormalization.
        let solved = CubeState::solved(4);
        let normalized_solved = solved.get_normalized();
        assert_eq!(solved.apply_algorithm("Rw Lw'").unwrap().get_normalized(), normalized_solved);
        assert_eq!(solved.apply_algorithm("Uw Dw'").unwrap().get_normalized(), normalized_solved);

        let threes = CubeState::solved(3);
        assert!(threes.apply("B").unwrap().equals_normalized(&threes.apply("Fw").unwrap()));

        let mut rng = rand::rng();
        let mut state = solved;
        for _ in 0..100 {
            let successors: Vec<CubeState> = state.successors_by_name().into_values().collect();
            state = successors[rng.random_range(0..successors.len())].clone();
            let rotated = state.apply_algorithm("Uw Dw'").unwrap();
            assert_eq!(state.get_normalized(), rotated.get_normalized());
        }
    }

    #[test]
    fn orientation_moves() {
        let threes = CubePuzzle::new(3).random_orientation_moves(1);
        assert_eq!(threes.len(), 24);
        assert!(threes.contains(&vec![]));
        assert!(threes.contains(&vec!["Rw".to_string(), "Uw'".to_string()]));
        let fours = CubePuzzle::new(4).random_orientation_moves(3);
        assert!(fours.contains(&vec!["x2".to_string(), "y".to_string()]));
    }

    #[test]
    fn apply_orientation_drops_redundant_moves() {
        // Port of NoInspectionFiveByFiveTest.
        let fives = CubePuzzle::new(5);
        assert_eq!(move_name(5, U, 1, 3).unwrap(), "4Uw");
        let reorient = vec!["4Uw".to_string()];
        let apply = |alg: &str| {
            let mut ab = AlgorithmBuilder::new(MergingMode::NoMerging, fives.solved_state());
            ab.append_algorithm(alg).unwrap();
            fives.apply_orientation(&reorient, ab.into_state_and_generator(), true).generator
        };
        assert_eq!(apply("F R"), "F R 4Uw");
        assert_eq!(apply("F D"), "F 4Uw");
        assert_eq!(apply("D U D U"), "U U 4Uw");
    }
}
