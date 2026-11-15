//! Clock scramble generation (WCA random-move format).

use rand::Rng;

/// Generate a WCA-standard Clock scramble.
///
/// Format: 9 pin-configuration turns on the front, `y2`, then 5 on the back.
/// Each turn is `0+`..`6+` or `1-`..`5-`.
pub fn generate_scramble(rng: &mut (impl Rng + ?Sized)) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Front side: 4 corner pins + 4 side pins + ALL, each with a random turn
    let front_names = ["UR", "DR", "DL", "UL", "U", "R", "D", "L", "ALL"];
    for &name in &front_names {
        let turn = rng.random_range(0..12) as i32 - 5; // -5 to +6
        if turn == 0 {
            parts.push(format!("{name}0+"));
        } else if turn > 0 {
            parts.push(format!("{name}{turn}+"));
        } else {
            parts.push(format!("{name}{}+", -turn).replace('+', "-"));
        }
    }

    parts.push("y2".to_string());

    // Back side: 4 side pins + ALL
    let back_names = ["U", "R", "D", "L", "ALL"];
    for &name in &back_names {
        let turn = rng.random_range(0..12) as i32 - 5;
        if turn == 0 {
            parts.push(format!("{name}0+"));
        } else if turn > 0 {
            parts.push(format!("{name}{turn}+"));
        } else {
            parts.push(format!("{name}{}+", -turn).replace('+', "-"));
        }
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_clock_scramble() {
        let mut rng = rand::rng();
        let scramble = generate_scramble(&mut rng);
        assert!(!scramble.is_empty());
        let moves: Vec<&str> = scramble.split_whitespace().collect();
        assert_eq!(moves.len(), 15, "{scramble}");
        assert_eq!(moves[9], "y2");
    }
}
