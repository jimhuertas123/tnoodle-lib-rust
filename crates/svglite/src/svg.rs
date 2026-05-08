use crate::dimension::Dimension;
use crate::element::Element;

/// Root `<svg>` element.
///
/// Matches Java's `org.worldcubeassociation.tnoodle.svglite.Svg`.
#[derive(Debug, Clone)]
pub struct Svg {
    pub size: Dimension,
    children: Vec<Element>,
}

impl Svg {
    pub fn new(size: Dimension) -> Self {
        Self {
            size,
            children: Vec::new(),
        }
    }

    pub fn append(&mut self, element: Element) {
        self.children.push(element);
    }

    pub fn children(&self) -> &[Element] {
        &self.children
    }

    /// Serialize the full SVG document to an XML string.
    pub fn to_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#,
            self.size.width, self.size.height
        ));
        s.push('\n');
        for child in &self.children {
            child.write_svg_to(&mut s, 1);
        }
        s.push_str("</svg>\n");
        s
    }
}

// Helper: Element needs a public-ish method for Svg to call.
// We add a crate-visible helper.
impl Element {
    pub(crate) fn write_svg_to(&self, s: &mut String, indent: usize) {
        // Reuse the private write_svg logic by calling to_svg_string
        // and indenting. But for efficiency, let's just delegate.
        let pad = "  ".repeat(indent);
        // We need to access write_svg, which is private.
        // Instead, let's make this work by making the SVG output inline.
        let raw = self.to_svg_string();
        for line in raw.lines() {
            s.push_str(&pad);
            s.push_str(line);
            s.push('\n');
        }
    }
}

impl std::fmt::Display for Svg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    #[test]
    fn basic_svg_document() {
        let mut svg = Svg::new(Dimension::new(100, 100));
        svg.append(Element::rect(0.0, 0.0, 100.0, 100.0).fill(Color::WHITE));
        let output = svg.to_string();
        assert!(output.starts_with("<svg"));
        assert!(output.contains("xmlns"));
        assert!(output.contains("</svg>"));
        assert!(output.contains(r##"fill="#ffffff""##));
    }
}
