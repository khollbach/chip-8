use crate::cpu::{
    io::DrawSprite,
    screen::{Point, DIMS},
};
use std::fmt::{self, Debug};

const WIDTH: usize = DIMS.x as usize;
const HEIGHT: usize = DIMS.y as usize;

#[derive(Clone)]
pub struct Screen {
    rows: [[bool; WIDTH]; HEIGHT],
}

impl Screen {
    pub fn new() -> Self {
        Self {
            rows: [[false; WIDTH]; HEIGHT],
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn draw_sprite(&mut self, top_left: Point, sprite: &[u8]) -> DrawSprite {
        let mut collision = false;

        for dy in 0..sprite.len() {
            for dx in 0..8 {
                let pos = top_left + (dx, dy as i8).into();

                // Quirk: ignore pixels that would wrap.
                // This causes sprites drawn at the borders to be "clipped".
                if !pos.in_bounds() {
                    continue;
                }

                let row: u8 = sprite[dy];
                let bit = 1 << (7 - dx);
                if row & bit != 0 {
                    if self.flip(pos) {
                        collision = true;
                    }
                }
            }
        }

        if collision {
            DrawSprite::Collision
        } else {
            DrawSprite::NoCollision
        }
    }

    /// Return true if there's a collision.
    fn flip(&mut self, p: Point) -> bool {
        assert!(p.in_bounds());

        let pixel = &mut self.rows[p.y as usize][p.x as usize];
        let was_high = *pixel;
        *pixel ^= true;

        was_high
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
