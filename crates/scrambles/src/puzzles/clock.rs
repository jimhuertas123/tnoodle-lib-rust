//! Clock scramble generation (WCA random-move format).

use rand::Rng;

/// Pin names for the scramble output.
const PIN_NAMES: [&str; 4] = ["UR", "DR", "DL", "UL"];

/// Generate a WCA-standard Clock scramble.
///
/// Format: pin positions + turns on front, y2, pin positions + turns on back,
/// then optional pin states.
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

    // Pin states (each corner pin is either up or down)
    for &pin in &PIN_NAMES {
        if rng.random_range(0..2) == 1 {
            parts.push(format!("{pin}"));
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
        assert!(scramble.contains("y2"));
    }
}
