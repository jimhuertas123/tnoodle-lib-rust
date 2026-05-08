//! Megaminx scramble generation (Pochmann-style random moves).

use rand::Rng;

/// Number of lines in a Megaminx scramble.
const HEIGHT: usize = 7;
/// Number of R/D moves per line.
const WIDTH: usize = 10;

/// Generate a WCA-standard Megaminx scramble.
///
/// Format: 7 lines of 10 R/D moves each, followed by U or U'.
/// R moves alternate between R++ and R--, D moves alternate between D++ and D--.
pub fn generate_scramble(rng: &mut (impl Rng + ?Sized)) -> String {
    let mut lines = Vec::with_capacity(HEIGHT);

    for _ in 0..HEIGHT {
        let mut line_moves = Vec::with_capacity(WIDTH + 1);

        for j in 0..WIDTH {
            if j % 2 == 0 {
                // R move
                if rng.random_range(0..2) == 0 {
                    line_moves.push("R++");
                } else {
                    line_moves.push("R--");
                }
            } else {
                // D move
                if rng.random_range(0..2) == 0 {
                    line_moves.push("D++");
                } else {
                    line_moves.push("D--");
                }
            }
        }

        // Final U move
        if rng.random_range(0..2) == 0 {
            line_moves.push("U");
        } else {
            line_moves.push("U'");
        }

        lines.push(line_moves.join(" "));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_mega_scramble() {
        let mut rng = rand::rng();
        let scramble = generate_scramble(&mut rng);
        let line_count = scramble.lines().count();
        assert_eq!(line_count, 7);
        for line in scramble.lines() {
            let move_count = line.split_whitespace().count();
            assert_eq!(move_count, 11); // 10 R/D + 1 U
        }
    }
}
