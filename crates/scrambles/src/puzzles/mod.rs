//! Concrete puzzle scramble generators for all WCA puzzle types.

pub mod big_cubes;
pub mod clock;
pub mod cube_puzzle;
pub mod cube_svg;
pub mod four_by_four;
pub mod megaminx;
pub mod pyraminx;
pub mod skewb;
pub mod square_one;
pub mod three_by_three;
pub mod two_by_two;

use crate::puzzle_registry::PuzzleType;
pub use tnoodle_min2phase::search::SearchBudget;
use tnoodle_random::JavaRandom;

/// How scrambles are generated (AD-014).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScrambleOptions {
    /// How long the 3x3 search (333, 333ni, 333fm) keeps looking for a shorter
    /// scramble. The default is tnoodle's: at least 200 ms, at most 60 s.
    pub search_budget: SearchBudget,
}

impl ScrambleOptions {
    /// Seeded scrambles stop the 3x3 search at its first solution, so a seed
    /// gives the same scramble on every machine (AD-012, `PARITY.md`).
    const SEEDED: Self = Self {
        search_budget: SearchBudget::Probes {
            min: 0,
            max: u64::MAX,
        },
    };
}

/// One WCA scramble drawn from `rng`. Port of `Puzzle.generateWcaScramble(Random)`.
pub fn generate_wca_scramble(
    puzzle_type: PuzzleType,
    rng: &mut JavaRandom,
    options: &ScrambleOptions,
) -> String {
    let budget = options.search_budget;
    match puzzle_type {
        PuzzleType::Two => two_by_two::generate_scramble(rng),
        PuzzleType::Three => three_by_three::generate_scramble(rng, budget),
        PuzzleType::Four => four_by_four::generate_scramble(rng),
        PuzzleType::FourFast => big_cubes::generate_scramble(4, rng),
        PuzzleType::Five => big_cubes::generate_scramble(5, rng),
        PuzzleType::Six => big_cubes::generate_scramble(6, rng),
        PuzzleType::Seven => big_cubes::generate_scramble(7, rng),
        PuzzleType::ThreeNi => three_by_three::generate_scramble_ni(rng, budget),
        PuzzleType::FourNi => four_by_four::generate_scramble_ni(rng),
        PuzzleType::FiveNi => big_cubes::generate_scramble_555ni(rng),
        PuzzleType::ThreeFm => three_by_three::generate_scramble_fm(rng, budget),
        PuzzleType::Pyra => pyraminx::generate_scramble(rng),
        PuzzleType::Sq1 => square_one::generate_scramble(rng),
        PuzzleType::Mega => megaminx::generate_scramble(rng),
        PuzzleType::Clock => clock::generate_scramble(rng),
        PuzzleType::Skewb => skewb::generate_scramble(rng),
    }
}

/// One WCA scramble from fresh randomness. Port of `Puzzle.generateScramble()`.
#[must_use]
pub fn generate_scramble(puzzle_type: PuzzleType) -> String {
    generate_scramble_with(puzzle_type, &ScrambleOptions::default())
}

/// One WCA scramble from fresh randomness, with `options`.
#[must_use]
pub fn generate_scramble_with(puzzle_type: PuzzleType, options: &ScrambleOptions) -> String {
    generate_wca_scramble(puzzle_type, &mut JavaRandom::from_entropy(), options)
}

/// `count` WCA scrambles from fresh randomness. Port of `Puzzle.generateScrambles(int)`.
#[must_use]
pub fn generate_scrambles(puzzle_type: PuzzleType, count: usize) -> Vec<String> {
    let mut rng = JavaRandom::from_entropy();
    let options = ScrambleOptions::default();
    (0..count)
        .map(|_| generate_wca_scramble(puzzle_type, &mut rng, &options))
        .collect()
}

/// The scramble tnoodle-lib gives for `seed`. Port of `Puzzle.generateSeededScramble(String)`.
#[must_use]
pub fn generate_seeded_scramble(puzzle_type: PuzzleType, seed: &str) -> String {
    generate_wca_scramble(
        puzzle_type,
        &mut JavaRandom::from_seed(seed.as_bytes()),
        &ScrambleOptions::SEEDED,
    )
}

/// The `count` scrambles tnoodle-lib gives for `seed`. Port of
/// `Puzzle.generateSeededScrambles(String, int)`.
#[must_use]
pub fn generate_seeded_scrambles(puzzle_type: PuzzleType, seed: &str, count: usize) -> Vec<String> {
    let mut rng = JavaRandom::from_seed(seed.as_bytes());
    (0..count)
        .map(|_| generate_wca_scramble(puzzle_type, &mut rng, &ScrambleOptions::SEEDED))
        .collect()
}

/// Generate SVG for a scramble visualization.
#[must_use]
pub fn generate_scramble_svg(puzzle_type: PuzzleType, scramble: &str) -> String {
    match puzzle_type {
        PuzzleType::Two => cube_svg::render_cube_svg(2, scramble),
        PuzzleType::Three | PuzzleType::ThreeNi | PuzzleType::ThreeFm => {
            cube_svg::render_cube_svg(3, scramble)
        }
        PuzzleType::Four | PuzzleType::FourFast | PuzzleType::FourNi => {
            cube_svg::render_cube_svg(4, scramble)
        }
        PuzzleType::Five | PuzzleType::FiveNi => cube_svg::render_cube_svg(5, scramble),
        PuzzleType::Six => cube_svg::render_cube_svg(6, scramble),
        PuzzleType::Seven => cube_svg::render_cube_svg(7, scramble),
        // Other puzzles: placeholder
        _ => String::new(),
    }
}
