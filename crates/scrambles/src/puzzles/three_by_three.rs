//! 3x3x3 scramble generation using the min2phase two-phase solver.
//!
//! Port of Java's `ThreeByThreeCubePuzzle`, `NoInspectionThreeByThreeCubePuzzle`
//! and `ThreeByThreeCubeFewestMovesPuzzle`.

use tnoodle_min2phase::search::{SearchBudget, SearchWca, Verbose};
use tnoodle_min2phase::tools;
use tnoodle_random::JavaRandom;

use super::cube_puzzle::{CubePuzzle, CubeState};
use crate::algorithm_builder::{AlgorithmBuilder, MergingMode};
use crate::puzzle::{Puzzle, PuzzleStateAndGenerator, solve_in};

/// Java's `THREE_BY_THREE_MAX_SCRAMBLE_LENGTH`.
const MAX_SCRAMBLE_LENGTH: i32 = 21;
const FM_PADDING: &str = "R' U' F";
/// Stops at the first solution, whatever it takes.
const FIRST_SOLUTION: SearchBudget = SearchBudget::Probes {
    min: 0,
    max: u64::MAX,
};

/// Random-state scramble whose underlying solution may not start on
/// `first_axis` nor end on `last_axis` (so the scramble may not end on
/// `first_axis` nor start on `last_axis`). Port of
/// `ThreeByThreeCubePuzzle.generateRandomMoves`.
fn generate_random_moves(
    rng: &mut JavaRandom,
    budget: SearchBudget,
    first_axis: Option<&str>,
    last_axis: Option<&str>,
) -> PuzzleStateAndGenerator<CubeState> {
    let mut search = SearchWca::new();
    let facelets = tools::random_cube(rng);
    let solve = |search: &mut SearchWca, budget| {
        search.solution(
            &facelets,
            MAX_SCRAMBLE_LENGTH,
            budget,
            Verbose::INVERSE_SOLUTION,
            first_axis,
            last_axis,
        )
    };
    // Java throws when the budget's maximum runs out before any solution
    // (PARITY.md); every valid state has one within 21 moves, so searching the
    // same state to its first solution always succeeds and draws nothing more.
    let scramble = solve(&mut search, budget)
        .or_else(|_| solve(&mut search, FIRST_SOLUTION))
        .expect("every 3x3 state has a solution within 21 moves");
    // Same as Java: canonicalize the trimmed solver output so it can never
    // contain redundant moves.
    let mut ab = AlgorithmBuilder::new(MergingMode::CanonicalizeMoves, CubeState::solved(3));
    ab.append_algorithm(scramble.trim())
        .expect("min2phase emits valid moves");
    ab.into_state_and_generator()
}

/// Reject scrambles that are too close to solved (Java `generateWcaScramble`).
fn wca_scramble(
    rng: &mut JavaRandom,
    mut generate: impl FnMut(&mut JavaRandom) -> PuzzleStateAndGenerator<CubeState>,
) -> String {
    let puzzle = CubePuzzle::new(3);
    loop {
        let psag = generate(rng);
        if solve_in(&puzzle, &psag.state, puzzle.wca_min_scramble_distance() - 1).is_none() {
            return psag.generator;
        }
    }
}

/// Generate a WCA-standard 3x3x3 scramble (random state, max 21 moves).
///
/// # Panics
///
/// Never: when `budget` runs out before any solution, the state is searched to
/// its first solution instead of failing as Java does.
pub fn generate_scramble(rng: &mut JavaRandom, budget: SearchBudget) -> String {
    wca_scramble(rng, |r| generate_random_moves(r, budget, None, None))
}

/// Generate a 3x3x3 blindfolded scramble: random state followed by a random
/// reorientation such as `Rw Uw'`. Port of `NoInspectionThreeByThreeCubePuzzle`:
/// the orientation is drawn before the state.
pub fn generate_scramble_ni(rng: &mut JavaRandom, budget: SearchBudget) -> String {
    let puzzle = CubePuzzle::new(3);
    let orientations = puzzle.random_orientation_moves(puzzle.size() / 2);
    wca_scramble(rng, |r| {
        let orientation = &orientations[r.next_index(orientations.len())];
        // The scramble must not end on the axis the reorientation starts with.
        let first_axis = orientation.first().map(|m| &m[..1]);
        let psag = generate_random_moves(r, budget, first_axis, None);
        puzzle.apply_orientation(orientation, psag, false)
    })
}

/// Generate a 3x3x3 Fewest Moves scramble, padded with `R' U' F` on both ends.
///
/// # Panics
///
/// Never: the padding and the solver's output are valid 3x3 moves.
pub fn generate_scramble_fm(rng: &mut JavaRandom, budget: SearchBudget) -> String {
    wca_scramble(rng, |r| {
        // The solution may not start on the suffix's first axis (R) nor end on
        // the prefix's last axis (F), so nothing cancels with the padding.
        let psag = generate_random_moves(r, budget, Some("R"), Some("F"));
        let mut ab = AlgorithmBuilder::new(MergingMode::NoMerging, CubeState::solved(3));
        for part in [FM_PADDING, psag.generator.as_str(), FM_PADDING] {
            ab.append_algorithm(part).expect("valid moves");
        }
        ab.into_state_and_generator()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::PuzzleState;
    use std::time::{Duration, Instant};

    /// The first solution min2phase finds, for tests whose claim is not about
    /// the search budget.
    const FAST: SearchBudget = SearchBudget::Probes {
        min: 0,
        max: 60_000,
    };

    #[test]
    fn time_budget_searches_at_least_min() {
        let mut rng = JavaRandom::from_seed(b"time budget");
        let budget = SearchBudget::Time {
            min: Duration::from_millis(200),
            max: Duration::from_secs(60),
        };
        for _ in 0..3 {
            let start = Instant::now();
            generate_scramble(&mut rng, budget);
            assert!(
                start.elapsed() >= Duration::from_millis(200),
                "took {:?}",
                start.elapsed()
            );
        }
    }

    #[test]
    fn probe_budget_zero_returns_first_solution() {
        // With no minimum, the search stops at the first solution: the same
        // one Java's SearchWCA returns with probeMin = 0.
        let mut search = SearchWca::new();
        let mut rng = JavaRandom::from_seed(b"first solution");
        let first_by_time = SearchBudget::Time {
            min: Duration::ZERO,
            max: Duration::from_secs(60),
        };
        for _ in 0..20 {
            let facelets = tools::random_cube(&mut rng);
            let by_probes =
                search.solution(&facelets, 21, FAST, Verbose::INVERSE_SOLUTION, None, None);
            let by_time = search.solution(
                &facelets,
                21,
                first_by_time,
                Verbose::INVERSE_SOLUTION,
                None,
                None,
            );
            assert_eq!(by_probes, by_time, "{facelets}");
        }
    }

    #[test]
    fn exhausted_budget_returns_first_solution_without_extra_draws() {
        type Generator = fn(&mut JavaRandom, SearchBudget) -> String;
        let none = SearchBudget::Time {
            min: Duration::ZERO,
            max: Duration::ZERO,
        };
        let generators: [Generator; 3] = [
            generate_scramble,
            generate_scramble_ni,
            generate_scramble_fm,
        ];
        for generate in generators {
            let mut exhausted = JavaRandom::from_seed(b"no budget");
            let mut first = JavaRandom::from_seed(b"no budget");
            for _ in 0..5 {
                let start = Instant::now();
                let scramble = generate(&mut exhausted, none);
                assert!(start.elapsed() < Duration::from_secs(5), "{scramble}");
                assert_eq!(scramble, generate(&mut first, FIRST_SOLUTION));
            }
            assert_eq!(exhausted.next_long(), first.next_long());
        }
    }

    #[test]
    fn average_length_matches_tnoodle() {
        // tnoodle's 3x3 scrambles average 18.6 moves with the default 200 ms budget.
        let mut rng = JavaRandom::from_seed(b"average length");
        let total: u32 = (0..300)
            .map(|_| {
                generate_scramble(&mut rng, SearchBudget::default())
                    .split_whitespace()
                    .count() as u32
            })
            .sum();
        let average = f64::from(total) / 300.0;
        assert!((18.0..=19.2).contains(&average), "average {average}");
    }

    #[test]
    fn generate_333_fm_scramble() {
        // Port of ThreeByThreeCubeFewestMovesTest: the padding never cancels,
        // so canonicalizing the scramble leaves it unchanged.
        let mut rng = JavaRandom::from_entropy();
        for _ in 0..20 {
            let scramble = generate_scramble_fm(&mut rng, FAST);
            assert!(scramble.starts_with("R' U' F "), "{scramble}");
            assert!(scramble.ends_with(" R' U' F"), "{scramble}");
            let mut ab =
                AlgorithmBuilder::new(MergingMode::CanonicalizeMoves, CubeState::solved(3));
            ab.append_algorithm(&scramble).unwrap();
            assert_eq!(ab.to_string(), scramble);
        }
    }

    #[test]
    fn generate_333ni_scramble() {
        let mut rng = JavaRandom::from_entropy();
        let mut saw_orientation = false;
        for _ in 0..30 {
            let scramble = generate_scramble_ni(&mut rng, FAST);
            let moves: Vec<&str> = scramble.split_whitespace().collect();
            let wide: Vec<&&str> = moves.iter().filter(|m| m.contains('w')).collect();
            assert!(wide.len() <= 2, "{scramble}");
            // Wide moves only appear as the trailing reorientation.
            assert!(
                moves[moves.len() - wide.len()..]
                    .iter()
                    .all(|m| m.contains('w')),
                "{scramble}"
            );
            saw_orientation |= !wide.is_empty();
            let state = CubeState::solved(3).apply_algorithm(&scramble).unwrap();
            assert!(!state.equals_normalized(&CubeState::solved(3)));
        }
        assert!(saw_orientation);
    }
}
