//! Clock scramble generation (WCA random-move format).

use tnoodle_random::JavaRandom;

/// One turn such as `UR3+` or `ALL2-`, as Java writes it: `0+` for no turn.
fn pin_move(name: &str, rng: &mut JavaRandom) -> String {
    let turn = i32::try_from(rng.next_index(12)).expect("below 12") - 5;
    let direction = if turn >= 0 { '+' } else { '-' };
    format!("{name}{}{direction}", turn.abs())
}

/// Generate a WCA-standard Clock scramble.
///
/// Format: 9 pin-configuration turns on the front, `y2`, then 5 on the back.
/// Each turn is `0+`..`6+` or `1-`..`5-`.
pub fn generate_scramble(rng: &mut JavaRandom) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Front side: 4 corner pins + 4 side pins + ALL, then the back side's 4 side
    // pins + ALL, each with a random turn from -5 to +6 (Java `nextInt(12) - 5`).
    for name in ["UR", "DR", "DL", "UL", "U", "R", "D", "L", "ALL"] {
        parts.push(pin_move(name, rng));
    }
    parts.push("y2".to_string());
    for name in ["U", "R", "D", "L", "ALL"] {
        parts.push(pin_move(name, rng));
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_clock_scramble() {
        let mut rng = JavaRandom::from_entropy();
        let scramble = generate_scramble(&mut rng);
        assert!(!scramble.is_empty());
        let moves: Vec<&str> = scramble.split_whitespace().collect();
        assert_eq!(moves.len(), 15, "{scramble}");
        assert_eq!(moves[9], "y2");
    }
}
