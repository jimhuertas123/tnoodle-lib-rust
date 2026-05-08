use tnoodle_scrambles::{PuzzleType, generate_scramble};

fn main() {
    let mut rng = rand::rng();

    let puzzles = [
        PuzzleType::Two,
        PuzzleType::Three,
        PuzzleType::Four,
        PuzzleType::Five,
        PuzzleType::Six,
        PuzzleType::Seven,
        PuzzleType::Sq1,
        PuzzleType::Pyra,
        PuzzleType::Skewb,
        PuzzleType::Mega,
        PuzzleType::Clock,
    ];

    for puzzle in puzzles {
        println!("--- {} ({}) ---", puzzle.long_name(), puzzle.short_name());
        let scramble = generate_scramble(puzzle, &mut rng);
        println!("{scramble}");
        println!();
    }
}
