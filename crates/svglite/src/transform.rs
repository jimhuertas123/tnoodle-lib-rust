use std::fmt;

/// 2D affine transform matrix:
/// ```text
/// | a  c  e |
/// | b  d  f |
/// | 0  0  1 |
/// ```
/// Matches Java's `org.worldcubeassociation.tnoodle.svglite.Transform`.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    pub fn translate(tx: f64, ty: f64) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: tx,
            f: ty,
        }
    }

    pub fn rotate(angle_rad: f64) -> Self {
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        }
    }

    pub fn scale(sx: f64, sy: f64) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Concatenate: self * other (apply other first, then self).
    pub fn concat(&self, other: &Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    /// Transform a point (x, y).
    pub fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }

    /// SVG `matrix(a,b,c,d,e,f)` string.
    pub fn to_svg_matrix(&self) -> String {
        format!(
            "matrix({},{},{},{},{},{})",
            self.a, self.b, self.c, self.d, self.e, self.f
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_svg_matrix())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_transform() {
        let t = Transform::IDENTITY;
        let (x, y) = t.apply(3.0, 4.0);
        assert!((x - 3.0).abs() < 1e-10);
        assert!((y - 4.0).abs() < 1e-10);
    }

    #[test]
    fn translate_transform() {
        let t = Transform::translate(10.0, 20.0);
        let (x, y) = t.apply(1.0, 2.0);
        assert!((x - 11.0).abs() < 1e-10);
        assert!((y - 22.0).abs() < 1e-10);
    }

    #[test]
    fn concat_transforms() {
        let t1 = Transform::translate(10.0, 0.0);
        let t2 = Transform::translate(0.0, 20.0);
        let combined = t1.concat(&t2);
        let (x, y) = combined.apply(0.0, 0.0);
        assert!((x - 10.0).abs() < 1e-10);
        assert!((y - 20.0).abs() < 1e-10);
    }
}
