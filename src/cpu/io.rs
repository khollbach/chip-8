use std::fmt::Debug;

use super::Point;

/// Nanosecond duration for the delay timer and sound timer "ticks".
///
/// Frequency: 60 Hz.
pub const TIME_BETWEEN_TICKS_NS: u64 = 10_u64.pow(9) / 60;

/// The input/output methods needed by the CHIP-8 interpreter.
pub trait Chip8Io: Debug {
    /// This method gets called periodically (currently after every instruction).
    ///
    /// You can use it to perform state updates, e.g. poll for keyboard input, etc.
    fn update(&mut self) {}

    fn clear_screen(&mut self);

    fn get_random_byte(&mut self) -> u8;

    /// Quirk: wait for the "display interrupt" (60 Hz) before returning.
    fn draw_sprite(&mut self, pos: Point, sprite: &[u8]) -> DrawSprite;

    /// Is the given key currently pressed? Keycodes are `0x0..=0xf`.
    fn is_key_pressed(&mut self, k: u8) -> bool;

    /// Block until any key gets pressed. Return that keycode.
    ///
    /// Quirk: this should wait until some key:
    /// 1. becomes newly pressed
    /// 2. later becomes released
    fn blocking_get_key(&mut self) -> u8;

    fn read_delay_timer(&mut self) -> u8;

    fn write_delay_timer(&mut self, value: u8);

    fn write_sound_timer(&mut self, value: u8);
}

pub enum DrawSprite {
    NoCollision,
    Collision,
}
