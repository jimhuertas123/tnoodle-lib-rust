//! Properties every generated scramble must satisfy, mirroring the checks in
//! Java's `HugeScrambleTest`, `AlgorithmBuilderTest` and the per-puzzle tests.

use std::collections::HashSet;

use tnoodle_scrambles::puzzles::cube_puzzle::{CubePuzzle, CubeState};
use tnoodle_scrambles::{
    AlgorithmBuilder, MergingMode, Puzzle, PuzzleState, PuzzleType, generate_scramble,
    puzzle::solve_in,
};

const SCRAMBLES_PER_PUZZLE: usize = 30;

/// Cube size, minimum distance from solved, and how many trailing moves are a
/// reorientation (blindfolded events) for each cube event.
fn cube_event(puzzle: PuzzleType) -> Option<(usize, i32, fn(&str) -> bool)> {
    fn none(_: &str) -> bool {
        false
    }
    fn wide(m: &str) -> bool {
        m.contains('w')
    }
    fn rotation(m: &str) -> bool {
        m.starts_with(['x', 'y', 'z'])
    }
    fn triple_wide(m: &str) -> bool {
        m.starts_with('3')
    }
    match puzzle {
        PuzzleType::Two => Some((2, 4, none)),
        PuzzleType::Three | PuzzleType::ThreeFm => Some((3, 2, none)),
        PuzzleType::ThreeNi => Some((3, 2, wide)),
        PuzzleType::Four | PuzzleType::FourFast => Some((4, 2, none)),
        PuzzleType::FourNi => Some((4, 2, rotation)),
        PuzzleType::Five => Some((5, 2, none)),
        PuzzleType::FiveNi => Some((5, 2, triple_wide)),
        PuzzleType::Six => Some((6, 2, none)),
        PuzzleType::Seven => Some((7, 2, none)),
        _ => None,
    }
}

fn axis(mv: &str) -> usize {
    let face = mv.chars().find(char::is_ascii_uppercase).unwrap();
    "URFDLB".find(face).unwrap() % 3
}

#[test]
fn cube_scrambles_are_valid_canonical_and_far_from_solved() {
    let mut rng = rand::rng();
    for &puzzle_type in PuzzleType::ALL {
        let Some((size, min_distance, is_reorientation)) = cube_event(puzzle_type) else {
            continue;
        };
        let puzzle = CubePuzzle::new(size);
        for _ in 0..SCRAMBLES_PER_PUZZLE {
            let scramble = generate_scramble(puzzle_type, &mut rng);
            let name = puzzle_type.short_name();

            // Valid moves, and not solvable in fewer than the WCA minimum.
            let state = puzzle
                .solved_state()
                .apply_algorithm(&scramble)
                .unwrap_or_else(|e| panic!("{name}: {e}: {scramble}"));
            assert!(
                solve_in(&puzzle, &state, min_distance - 1).is_none(),
                "{name} scramble within {} moves of solved: {scramble}",
                min_distance - 1
            );

            // Canonicalizing the scramble (without the reorientation suffix)
            // must not change it: no move cancels or merges with an earlier
            // move it commutes with, so no `R R`, `R L R`, `U D U'`, `Lw` on 4x4...
            let moves: Vec<&str> = scramble.split_whitespace().collect();
            let suffix = moves
                .iter()
                .rev()
                .take_while(|m| is_reorientation(m))
                .count();
            let core = moves[..moves.len() - suffix].join(" ");
            let mut ab =
                AlgorithmBuilder::new(MergingMode::CanonicalizeMoves, CubeState::solved(size));
            ab.append_algorithm(&core).unwrap();
            assert_eq!(ab.to_string(), core, "{name} scramble is not canonical");

            let core_moves = &moves[..moves.len() - suffix];
            for w in core_moves.windows(3) {
                let same_face =
                    w[0].trim_end_matches(['2', '\'']) == w[2].trim_end_matches(['2', '\'']);
                assert!(
                    !(same_face && axis(w[0]) == axis(w[1])),
                    "{name} redundant {} {} {}: {scramble}",
                    w[0],
                    w[1],
                    w[2]
                );
            }
            if size % 2 == 0 {
                let half = if size == 4 { "" } else { "3" };
                for face in ["L", "D", "B"] {
                    let banned = format!("{half}{face}w");
                    assert!(
                        !core_moves.iter().any(|m| m.starts_with(&banned)),
                        "{name} uses {banned}: {scramble}"
                    );
                }
            }
            if size == 3 && puzzle_type != PuzzleType::ThreeFm {
                assert!(
                    core_moves.len() <= 21,
                    "{name} longer than 21 moves: {scramble}"
                );
            }
        }
    }
}

#[test]
fn every_puzzle_generates_scrambles() {
    // Port of HugeScrambleTest.testNames plus a smoke test for each puzzle.
    let mut rng = rand::rng();
    let names: HashSet<&str> = PuzzleType::ALL.iter().map(PuzzleType::short_name).collect();
    assert_eq!(names.len(), PuzzleType::ALL.len());
    for &puzzle_type in PuzzleType::ALL {
        for _ in 0..5 {
            let scramble = generate_scramble(puzzle_type, &mut rng);
            assert!(!scramble.trim().is_empty(), "{}", puzzle_type.short_name());
        }
    }
}
