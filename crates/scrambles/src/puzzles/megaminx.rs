//! Megaminx scramble generation (Pochmann-style random moves).

use tnoodle_random::JavaRandom;

/// Number of lines in a Megaminx scramble.
const HEIGHT: usize = 7;
/// Number of R/D moves per line.
const WIDTH: usize = 10;

/// Generate a WCA-standard Megaminx scramble.
///
/// Format: 7 lines of 10 R/D moves each, followed by U or U'.
/// Each R/D turn is randomly `++` or `--`; the U turn matches the last D turn.
pub fn generate_scramble(rng: &mut JavaRandom) -> String {
    let mut lines = Vec::with_capacity(HEIGHT);

    for _ in 0..HEIGHT {
        let mut line_moves = Vec::with_capacity(WIDTH + 1);
        let mut clockwise = true;

        for j in 0..WIDTH {
            let side = if j % 2 == 0 { "R" } else { "D" };
            clockwise = rng.next_index(2) == 0;
            line_moves.push(format!("{side}{}", if clockwise { "++" } else { "--" }));
        }

        // Pochmann style: the final U turn follows the direction of the last D turn.
        line_moves.push(if clockwise { "U" } else { "U'" }.to_string());

        lines.push(line_moves.join(" "));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_mega_scramble() {
        let mut rng = JavaRandom::from_entropy();
        let scramble = generate_scramble(&mut rng);
        let line_count = scramble.lines().count();
        assert_eq!(line_count, 7);
        for line in scramble.lines() {
            let move_count = line.split_whitespace().count();
            assert_eq!(move_count, 11); // 10 R/D + 1 U
            let moves: Vec<&str> = line.split_whitespace().collect();
            let expected_u = if moves[9] == "D++" { "U" } else { "U'" };
            assert_eq!(moves[10], expected_u, "{line}");
        }
    }
}
