use std::fmt;

/// RGBA color, matching Java's `org.worldcubeassociation.tnoodle.svglite.Color`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parse a hex color string: "#RRGGBB" or "#RRGGBBAA".
    pub fn from_hex(hex: &str) -> Result<Self, InvalidHexColorError> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| InvalidHexColorError(hex.to_string()))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| InvalidHexColorError(hex.to_string()))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| InvalidHexColorError(hex.to_string()))?;
                Ok(Self::new(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| InvalidHexColorError(hex.to_string()))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| InvalidHexColorError(hex.to_string()))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| InvalidHexColorError(hex.to_string()))?;
                let a = u8::from_str_radix(&hex[6..8], 16)
                    .map_err(|_| InvalidHexColorError(hex.to_string()))?;
                Ok(Self::with_alpha(r, g, b, a))
            }
            _ => Err(InvalidHexColorError(hex.to_string())),
        }
    }

    /// Return the hex string "#rrggbb" (lowercase).
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }

    // Common puzzle colors
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const YELLOW: Self = Self::new(255, 255, 0);
    pub const RED: Self = Self::new(255, 0, 0);
    pub const ORANGE: Self = Self::new(255, 165, 0);
    pub const BLUE: Self = Self::new(0, 0, 255);
    pub const GREEN: Self = Self::new(0, 128, 0);
    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const GRAY: Self = Self::new(128, 128, 128);
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

#[derive(Debug, Clone)]
pub struct InvalidHexColorError(pub String);

impl fmt::Display for InvalidHexColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid hex color: {}", self.0)
    }
}

impl std::error::Error for InvalidHexColorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_color() {
        let c = Color::from_hex("#ff8000").unwrap();
        assert_eq!(c, Color::new(255, 128, 0));
        assert_eq!(c.to_hex(), "#ff8000");
    }

    #[test]
    fn parse_hex_with_alpha() {
        let c = Color::from_hex("#ff800080").unwrap();
        assert_eq!(c, Color::with_alpha(255, 128, 0, 128));
    }

    #[test]
    fn constants() {
        assert_eq!(Color::RED.to_hex(), "#ff0000");
        assert_eq!(Color::WHITE.to_hex(), "#ffffff");
    }
}
