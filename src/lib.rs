mod cpu;
mod terminal_io;

use cpu::{io::Chip8Io, Chip8};

pub use terminal_io::TerminalIo;

pub fn run(rom: &[u8], io: &mut dyn Chip8Io) {
    Chip8::new(rom, io).run()
}
