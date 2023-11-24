use std::{
    fmt::{self, Debug},
    ops::{Index, IndexMut},
};

use crate::debug::{self, DebugHexByte};

#[derive(Clone)]
pub struct Mem {
    bytes: Box<[u8; Self::LEN as usize]>,
}

impl Mem {
    pub const LEN: u16 = 4 * 1024;
    pub const ROM_START: u16 = 0x0200;

    /// Load a ROM into memory, starting at offset 0x200.
    pub fn new(rom: &[u8]) -> Self {
        let rom_start = Self::ROM_START as usize;
        assert!(rom_start + rom.len() <= Self::LEN as usize);

        let mut bytes = Box::new([0u8; Self::LEN as usize]);
        bytes[rom_start..][..rom.len()].copy_from_slice(rom);

        // Load built-in sprites into memory starting at offset 0x000.
        let digits_rom: Vec<_> = DIGITS.into_iter().flatten().collect();
        bytes[..digits_rom.len()].copy_from_slice(&digits_rom);

        Self { bytes }
    }
}

/// Bitmaps for the built-in hex digit sprites.
const DIGITS: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0],
    [0x20, 0x60, 0x20, 0x20, 0x70],
    [0xF0, 0x10, 0xF0, 0x80, 0xF0],
    [0xF0, 0x10, 0xF0, 0x10, 0xF0],
    [0x90, 0x90, 0xF0, 0x10, 0x10],
    [0xF0, 0x80, 0xF0, 0x10, 0xF0],
    [0xF0, 0x80, 0xF0, 0x90, 0xF0],
    [0xF0, 0x10, 0x20, 0x40, 0x40],
    [0xF0, 0x90, 0xF0, 0x90, 0xF0],
    [0xF0, 0x90, 0xF0, 0x10, 0xF0],
    [0xF0, 0x90, 0xF0, 0x90, 0x90],
    [0xE0, 0x90, 0xE0, 0x90, 0xE0],
    [0xF0, 0x80, 0x80, 0x80, 0xF0],
    [0xE0, 0x90, 0x90, 0x90, 0xE0],
    [0xF0, 0x80, 0xF0, 0x80, 0xF0],
    [0xF0, 0x80, 0xF0, 0x80, 0x80],
];

impl Index<u16> for Mem {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.bytes[index as usize]
    }
}

impl IndexMut<u16> for Mem {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.bytes[index as usize]
    }
}

impl Debug for Mem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return self.debug_compact(f);
        }

        // Similar to unix `hexdump`: print rows of bytes.

        writeln!(f)?;

        let mut prev_blank = false;
        for (i, line) in self.bytes.chunks(16).enumerate() {
            // Skip large blocks of zeros.
            if line == [0; 16] {
                // Print an indication at the start of the block.
                if !prev_blank {
                    writeln!(f, "...")?;
                }
                prev_blank = true;
                continue;
            }
            prev_blank = false;

            write!(f, "{i:02x}0: ")?;
            debug::write_row(f, line.try_into().unwrap())?;
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Mem {
    /// Helper for <Mem as Debug>::fmt
    fn debug_compact(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show up to and including the last non-zero value.
        // I.e., skip the suffix of all zeros.
        let last_nonzero = self.bytes.iter().enumerate().rev().find(|&(_i, &x)| x != 0);
        let end_idx = last_nonzero.map(|(i, _x)| i + 1).unwrap_or(0);
        let entries = self.bytes[..end_idx].iter().copied().map(DebugHexByte);

        f.debug_list().entries(entries).finish()
    }
}
