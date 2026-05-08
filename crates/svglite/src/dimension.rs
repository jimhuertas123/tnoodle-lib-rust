/// 2D integer dimensions (width, height).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimension {
    pub width: i32,
    pub height: i32,
}

impl Dimension {
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}
