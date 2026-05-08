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
pub mod cube_svg;

use rand::Rng;
use crate::puzzle_registry::PuzzleType;

/// Merge consecutive same-face moves in a scramble string.
///
/// E.g., "U' U'" → "U2", "R R" → "R2", "R R'" → "" (cancel), "R R2" → "R'"
pub fn clean_scramble(scramble: &str) -> String {
    let tokens: Vec<&str> = scramble.split_whitespace().collect();
    if tokens.is_empty() {
        return String::new();
    }

    // Parse each token into (base, quarter_turns_cw)
    // base = everything except the direction suffix (e.g., "R", "Rw", "3Rw")
    // quarter_turns = 1 (CW), 2 (180), 3 (CCW)
    let parsed: Vec<(&str, String, i32)> = tokens.iter().map(|&t| {
        let (base, turns) = if t.ends_with('\'') || t.ends_with('\u{2019}') {
            (&t[..t.len() - t.chars().last().unwrap().len_utf8()], 3)
        } else if t.ends_with('2') {
            (&t[..t.len() - 1], 2)
        } else {
            (t, 1)
        };
        (t, base.to_string(), turns)
    }).collect();

    let mut result: Vec<(String, i32)> = Vec::new(); // (base, accumulated_turns mod 4)

    for (_orig, base, turns) in &parsed {
        if let Some(last) = result.last_mut() {
            if last.0 == *base {
                // Same base move — merge
                last.1 = (last.1 + turns) % 4;
                continue;
            }
        }
        result.push((base.clone(), *turns));
    }

    // Convert back to strings, dropping zero-turn entries
    let mut output = Vec::new();
    for (base, turns) in &result {
        match turns % 4 {
            0 => {} // cancelled out
            1 => output.push(base.clone()),
            2 => output.push(format!("{base}2")),
            3 => output.push(format!("{base}'")),
            _ => unreachable!(),
        }
    }

    // Second pass: check if merging created new adjacent same-base pairs
    // (e.g., "R U U' R'" after merging U+U' cancels, leaving "R R'" which should also merge)
    let joined = output.join(" ");
    if output.len() < tokens.len() && output.len() > 1 {
        // Recurse to handle cascading merges
        clean_scramble(&joined)
    } else {
        joined
    }
}

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
        PuzzleType::FourNi => four_by_four::generate_scramble(rng),
        PuzzleType::FiveNi => big_cubes::generate_scramble(5, rng),
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
