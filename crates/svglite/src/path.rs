use std::fmt;

/// SVG path command.
#[derive(Debug, Clone, Copy)]
pub enum PathCommand {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    Close,
}

/// SVG `<path>` element builder.
///
/// Matches Java's `org.worldcubeassociation.tnoodle.svglite.Path`.
#[derive(Debug, Clone)]
pub struct Path {
    commands: Vec<PathCommand>,
}

impl Path {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.commands.push(PathCommand::MoveTo(x, y));
        self
    }

    pub fn line_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.commands.push(PathCommand::LineTo(x, y));
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.commands.push(PathCommand::Close);
        self
    }

    pub fn commands(&self) -> &[PathCommand] {
        &self.commands
    }

    /// Build the SVG `d` attribute string.
    pub fn to_d_string(&self) -> String {
        let mut s = String::new();
        for cmd in &self.commands {
            if !s.is_empty() {
                s.push(' ');
            }
            match cmd {
                PathCommand::MoveTo(x, y) => {
                    s.push_str(&format!("M {x} {y}"));
                }
                PathCommand::LineTo(x, y) => {
                    s.push_str(&format!("L {x} {y}"));
                }
                PathCommand::Close => {
                    s.push('Z');
                }
            }
        }
        s
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_d_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_path() {
        let mut p = Path::new();
        p.move_to(0.0, 0.0).line_to(10.0, 0.0).line_to(5.0, 10.0).close();
        assert_eq!(p.to_d_string(), "M 0 0 L 10 0 L 5 10 Z");
    }
}
