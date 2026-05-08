use std::collections::HashMap;
use std::fmt;

use crate::color::Color;
use crate::path::Path;
use crate::transform::Transform;

/// An SVG element node. Uses an enum for the fixed set of element types
/// rather than the Java class hierarchy.
#[derive(Debug, Clone)]
pub enum Element {
    Group {
        attrs: Attrs,
        children: Vec<Element>,
    },
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        attrs: Attrs,
    },
    Circle {
        cx: f64,
        cy: f64,
        r: f64,
        attrs: Attrs,
    },
    Ellipse {
        cx: f64,
        cy: f64,
        rx: f64,
        ry: f64,
        attrs: Attrs,
    },
    PathEl {
        path: Path,
        attrs: Attrs,
    },
    Text {
        x: f64,
        y: f64,
        content: String,
        attrs: Attrs,
    },
}

/// Common SVG attributes and styles for any element.
#[derive(Debug, Clone, Default)]
pub struct Attrs {
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub stroke_width: Option<f64>,
    pub opacity: Option<f64>,
    pub transform: Option<Transform>,
    pub extra: HashMap<String, String>,
    pub styles: HashMap<String, String>,
}

impl Attrs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    pub fn with_stroke(mut self, color: Color) -> Self {
        self.stroke = Some(color);
        self
    }

    pub fn with_stroke_width(mut self, w: f64) -> Self {
        self.stroke_width = Some(w);
        self
    }

    pub fn with_transform(mut self, t: Transform) -> Self {
        self.transform = Some(t);
        self
    }

    pub fn with_opacity(mut self, o: f64) -> Self {
        self.opacity = Some(o);
        self
    }

    fn write_attrs(&self, s: &mut String) {
        if let Some(ref fill) = self.fill {
            s.push_str(&format!(r#" fill="{fill}""#));
        }
        if let Some(ref stroke) = self.stroke {
            s.push_str(&format!(r#" stroke="{stroke}""#));
        }
        if let Some(w) = self.stroke_width {
            s.push_str(&format!(r#" stroke-width="{w}""#));
        }
        if let Some(o) = self.opacity {
            s.push_str(&format!(r#" opacity="{o}""#));
        }
        if let Some(ref t) = self.transform {
            s.push_str(&format!(r#" transform="{t}""#));
        }
        for (k, v) in &self.extra {
            s.push_str(&format!(r#" {k}="{v}""#));
        }
        if !self.styles.is_empty() {
            let style_str: String = self
                .styles
                .iter()
                .map(|(k, v)| format!("{k}:{v}"))
                .collect::<Vec<_>>()
                .join(";");
            s.push_str(&format!(r#" style="{style_str}""#));
        }
    }
}

impl Element {
    pub fn group() -> Self {
        Self::Group {
            attrs: Attrs::new(),
            children: Vec::new(),
        }
    }

    pub fn rect(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self::Rect {
            x,
            y,
            width,
            height,
            attrs: Attrs::new(),
        }
    }

    pub fn circle(cx: f64, cy: f64, r: f64) -> Self {
        Self::Circle {
            cx,
            cy,
            r,
            attrs: Attrs::new(),
        }
    }

    pub fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64) -> Self {
        Self::Ellipse {
            cx,
            cy,
            rx,
            ry,
            attrs: Attrs::new(),
        }
    }

    pub fn path(p: Path) -> Self {
        Self::PathEl {
            path: p,
            attrs: Attrs::new(),
        }
    }

    pub fn text(x: f64, y: f64, content: impl Into<String>) -> Self {
        Self::Text {
            x,
            y,
            content: content.into(),
            attrs: Attrs::new(),
        }
    }

    /// Get a mutable reference to this element's attrs.
    pub fn attrs_mut(&mut self) -> &mut Attrs {
        match self {
            Self::Group { attrs, .. }
            | Self::Rect { attrs, .. }
            | Self::Circle { attrs, .. }
            | Self::Ellipse { attrs, .. }
            | Self::PathEl { attrs, .. }
            | Self::Text { attrs, .. } => attrs,
        }
    }

    /// Set fill color, builder-style.
    pub fn fill(mut self, color: Color) -> Self {
        self.attrs_mut().fill = Some(color);
        self
    }

    /// Set stroke color, builder-style.
    pub fn stroke(mut self, color: Color) -> Self {
        self.attrs_mut().stroke = Some(color);
        self
    }

    /// Set stroke width, builder-style.
    pub fn stroke_width(mut self, w: f64) -> Self {
        self.attrs_mut().stroke_width = Some(w);
        self
    }

    /// Set transform, builder-style.
    pub fn transform(mut self, t: Transform) -> Self {
        self.attrs_mut().transform = Some(t);
        self
    }

    /// Append a child element (only works on Group).
    pub fn append(&mut self, child: Element) {
        if let Self::Group { children, .. } = self {
            children.push(child);
        }
    }

    /// Serialize to SVG XML string.
    pub fn to_svg_string(&self) -> String {
        let mut s = String::new();
        self.write_svg(&mut s, 0);
        s
    }

    fn write_svg(&self, s: &mut String, indent: usize) {
        let pad = "  ".repeat(indent);
        match self {
            Self::Group { attrs, children } => {
                s.push_str(&format!("{pad}<g"));
                attrs.write_attrs(s);
                if children.is_empty() {
                    s.push_str("/>\n");
                } else {
                    s.push_str(">\n");
                    for child in children {
                        child.write_svg(s, indent + 1);
                    }
                    s.push_str(&format!("{pad}</g>\n"));
                }
            }
            Self::Rect {
                x,
                y,
                width,
                height,
                attrs,
            } => {
                s.push_str(&format!(
                    r#"{pad}<rect x="{x}" y="{y}" width="{width}" height="{height}""#
                ));
                attrs.write_attrs(s);
                s.push_str("/>\n");
            }
            Self::Circle { cx, cy, r, attrs } => {
                s.push_str(&format!(r#"{pad}<circle cx="{cx}" cy="{cy}" r="{r}""#));
                attrs.write_attrs(s);
                s.push_str("/>\n");
            }
            Self::Ellipse {
                cx,
                cy,
                rx,
                ry,
                attrs,
            } => {
                s.push_str(&format!(
                    r#"{pad}<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}""#
                ));
                attrs.write_attrs(s);
                s.push_str("/>\n");
            }
            Self::PathEl { path, attrs } => {
                s.push_str(&format!(r#"{pad}<path d="{path}""#));
                attrs.write_attrs(s);
                s.push_str("/>\n");
            }
            Self::Text {
                x,
                y,
                content,
                attrs,
            } => {
                s.push_str(&format!(r#"{pad}<text x="{x}" y="{y}""#));
                attrs.write_attrs(s);
                s.push_str(&format!(">{content}</text>\n"));
            }
        }
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_svg_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_with_fill() {
        let r = Element::rect(0.0, 0.0, 10.0, 10.0).fill(Color::RED);
        let svg = r.to_svg_string();
        assert!(svg.contains(r##"fill="#ff0000""##));
        assert!(svg.contains(r#"width="10""#));
    }

    #[test]
    fn group_with_children() {
        let mut g = Element::group();
        g.append(Element::rect(0.0, 0.0, 5.0, 5.0).fill(Color::BLUE));
        g.append(Element::circle(10.0, 10.0, 3.0).fill(Color::GREEN));
        let svg = g.to_svg_string();
        assert!(svg.contains("<g>"));
        assert!(svg.contains("</g>"));
        assert!(svg.contains("<rect"));
        assert!(svg.contains("<circle"));
    }
}
