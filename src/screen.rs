use std::fmt::{self, Debug};

#[derive(Clone)]
pub struct Screen {
    rows: [[bool; Self::WIDTH]; Self::HEIGHT],
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i8,
    pub y: i8,
}

impl Screen {
    const DIMS: Point = Point { x: 64, y: 32 };
    const WIDTH: usize = Self::DIMS.x as usize;
    const HEIGHT: usize = Self::DIMS.y as usize;

    fn new() -> Self {
        Self {
            rows: [[false; Self::WIDTH]; Self::HEIGHT],
        }
    }

    pub fn flip(&mut self, p: Point) -> Flip {
        assert!(p.in_bounds());

        let pixel = &mut self.rows[p.y as usize][p.x as usize];
        let was_high = *pixel;
        *pixel ^= true;

        if was_high {
            Flip::Collision
        } else {
            Flip::NoCollision
        }
    }
}

pub enum Flip {
    NoCollision,
    Collision,
}

impl Default for Screen {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for y in 0..self.rows.len() {
            for x in 0..self.rows[y].len() {
                let c = if self.rows[y][x] { '#' } else { '.' };
                write!(f, "{c}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Point {
    pub fn wrapping_add(self, other: Self) -> Self {
        Self {
            x: (self.x + other.x).rem_euclid(Screen::DIMS.x),
            y: (self.y + other.y).rem_euclid(Screen::DIMS.y),
        }
    }

    fn in_bounds(self) -> bool {
        let x = 0 <= self.x && self.x < Screen::DIMS.x;
        let y = 0 <= self.y && self.y < Screen::DIMS.y;
        x && y
    }
}

impl From<(i8, i8)> for Point {
    fn from((x, y): (i8, i8)) -> Self {
        Self { x, y }
    }
}
