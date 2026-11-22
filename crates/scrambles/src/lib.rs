//! Rust port of tnoodle-lib's scramble generation (`org.worldcubeassociation.tnoodle`).
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    reason = "mirrors Java's int index arithmetic over sticker and move tables, each bounded by the cube size"
)]

pub mod algorithm_builder;
pub mod puzzle;
pub mod puzzle_registry;
pub mod puzzles;
pub mod scramble_cacher;

// Re-export main types
pub use algorithm_builder::{AlgorithmBuilder, MergingMode};
pub use puzzle::{Puzzle, PuzzleError, PuzzleState, PuzzleStateAndGenerator};
pub use puzzle_registry::PuzzleType;
pub use puzzles::{
    ScrambleOptions, SearchBudget, generate_scramble, generate_scramble_svg,
    generate_scramble_with, generate_scrambles, generate_seeded_scramble,
    generate_seeded_scrambles, generate_wca_scramble,
};
pub use scramble_cacher::ScrambleCacher;
pub use tnoodle_random::JavaRandom;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_scrambles_repeat_for_every_event() {
        for &puzzle in PuzzleType::ALL {
            let first = generate_seeded_scramble(puzzle, "repeat");
            assert_eq!(
                generate_seeded_scramble(puzzle, "repeat"),
                first,
                "{}",
                puzzle.short_name()
            );
            assert!(!first.trim().is_empty(), "{}", puzzle.short_name());
        }
    }

    #[test]
    fn concurrent_seeded_scrambles_match_sequential() {
        let puzzles = [
            PuzzleType::Two,
            PuzzleType::Three,
            PuzzleType::Four,
            PuzzleType::Five,
            PuzzleType::Sq1,
            PuzzleType::Skewb,
        ];
        let seeds: Vec<String> = (0..8).map(|i| format!("thread {i}")).collect();
        let sequential: Vec<Vec<String>> = seeds
            .iter()
            .map(|seed| {
                puzzles
                    .iter()
                    .map(|&p| generate_seeded_scramble(p, seed))
                    .collect()
            })
            .collect();
        let concurrent: Vec<Vec<String>> = std::thread::scope(|scope| {
            let handles: Vec<_> = seeds
                .iter()
                .map(|seed| {
                    scope.spawn(move || {
                        puzzles
                            .iter()
                            .map(|&p| generate_seeded_scramble(p, seed))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().expect("thread"))
                .collect()
        });
        assert_eq!(concurrent, sequential);
    }
}
