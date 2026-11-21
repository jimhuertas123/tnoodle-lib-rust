//! Concrete puzzle scramble generators for all WCA puzzle types.

pub mod three_by_three;
pub mod two_by_two;
pub mod four_by_four;
pub mod big_cubes;
pub mod square_one;
pub mod megaminx;
pub mod pyraminx;
pub mod skewb;
pub mod clock;
pub mod cube_puzzle;
pub mod cube_svg;

use rand::Rng;
use crate::puzzle_registry::PuzzleType;

/// Generate a WCA-legal scramble string for the given puzzle type.
pub fn generate_scramble(puzzle_type: PuzzleType, rng: &mut impl Rng) -> String {
    match puzzle_type {
        PuzzleType::Two => two_by_two::generate_scramble(rng),
        PuzzleType::Three => three_by_three::generate_scramble(rng),
        PuzzleType::Four => four_by_four::generate_scramble(rng),
        PuzzleType::FourFast => big_cubes::generate_scramble(4, rng),
        PuzzleType::Five => big_cubes::generate_scramble(5, rng),
        PuzzleType::Six => big_cubes::generate_scramble(6, rng),
        PuzzleType::Seven => big_cubes::generate_scramble(7, rng),
        PuzzleType::ThreeNi => three_by_three::generate_scramble_ni(rng),
        PuzzleType::FourNi => four_by_four::generate_scramble_ni(rng),
        PuzzleType::FiveNi => big_cubes::generate_scramble_555ni(rng),
        PuzzleType::ThreeFm => three_by_three::generate_scramble_fm(rng),
        PuzzleType::Pyra => pyraminx::generate_scramble(rng),
        PuzzleType::Sq1 => square_one::generate_scramble(rng),
        PuzzleType::Mega => megaminx::generate_scramble(rng),
        PuzzleType::Clock => clock::generate_scramble(rng),
        PuzzleType::Skewb => skewb::generate_scramble(rng),
    }
}

/// Generate SVG for a scramble visualization.
pub fn generate_scramble_svg(puzzle_type: PuzzleType, scramble: &str) -> String {
    match puzzle_type {
        PuzzleType::Two => cube_svg::render_cube_svg(2, scramble),
        PuzzleType::Three
        | PuzzleType::ThreeNi
        | PuzzleType::ThreeFm => cube_svg::render_cube_svg(3, scramble),
        PuzzleType::Four
        | PuzzleType::FourFast
        | PuzzleType::FourNi => cube_svg::render_cube_svg(4, scramble),
        PuzzleType::Five
        | PuzzleType::FiveNi => cube_svg::render_cube_svg(5, scramble),
        PuzzleType::Six => cube_svg::render_cube_svg(6, scramble),
        PuzzleType::Seven => cube_svg::render_cube_svg(7, scramble),
        // Other puzzles: placeholder
        _ => String::new(),
    }
}
