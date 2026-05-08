pub mod algorithm_builder;
pub mod puzzle;
pub mod puzzle_registry;
pub mod puzzles;
pub mod scramble_cacher;

// Re-export main types
pub use algorithm_builder::{AlgorithmBuilder, MergingMode};
pub use puzzle::{Puzzle, PuzzleError, PuzzleState, PuzzleStateAndGenerator};
pub use puzzle_registry::PuzzleType;
pub use puzzles::generate_scramble;
pub use puzzles::generate_scramble_svg;
pub use scramble_cacher::ScrambleCacher;
