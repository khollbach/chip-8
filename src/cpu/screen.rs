use std::ops::Add;

pub const DIMS: Point = Point { x: 64, y: 32 };

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i8,
    pub y: i8,
}

impl From<(i8, i8)> for Point {
    fn from((x, y): (i8, i8)) -> Self {
        Self { x, y }
    }
}

impl Point {
    #[must_use]
    pub fn wrap(self) -> Self {
        Self {
            x: self.x.rem_euclid(DIMS.x),
            y: self.y.rem_euclid(DIMS.y),
        }
    }

    pub fn in_bounds(self) -> bool {
        let x = 0 <= self.x && self.x < DIMS.x;
        let y = 0 <= self.y && self.y < DIMS.y;
        x && y
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
