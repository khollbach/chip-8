mod debug;
mod mem;
mod regs;
mod screen;
mod stack;

use mem::Mem;
use regs::Regs;
use screen::Screen;
use stack::Stack;

use crate::screen::{Flip, Point};

#[derive(Debug, Clone)]
pub struct Chip8 {
    pc: u16,
    i: u16,
    stack: Stack,
    v: Regs,
    mem: Mem,
    screen: Screen,
}

impl Chip8 {
    pub fn new(rom: &[u8]) -> Self {
        Self {
            pc: Mem::ROM_START,
            i: 0,
            stack: Stack::new(),
            v: Regs::new(),
            mem: Mem::new(rom),
            screen: Screen::default(),
        }
    }

    pub fn step(&mut self) {
        debug_assert!(self.pc < Mem::LEN);
        debug_assert_eq!(self.pc % 2, 0);

        let j = self.mem[self.pc];
        let k = self.mem[self.pc + 1];
        let instr = u16::from_be_bytes([j, k]);
        let pc = self.pc;
        let err = move || panic!("unimplemented: 0x{instr:04x} (pc=0x{pc:04x})");
        self.pc += 2;

        let [op, x, y, n] = nibbles_from_u16(instr);
        let addr = instr & 0x0fff;

        match op {
            0x0 => match instr {
                0x00e0 => self.screen = Screen::default(),
                0x00ee => self.pc = self.stack.pop(),
                _ => err(),
            },
            0x1 => self.pc = addr,
            0x2 => {
                self.stack.push(self.pc);
                self.pc = addr;
            }
            0x3 => {
                if self.v[x] == k {
                    self.pc += 2;
                }
            }
            0x4 => {
                if self.v[x] != k {
                    self.pc += 2;
                }
            }
            0x5 => {
                assert_eq!(n, 0);
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            0x6 => self.v[x] = k,
            0x7 => self.v[x] = self.v[x].wrapping_add(k),
            0x8 => match n {
                0x0 => self.v[x] = self.v[y],
                0x1 => self.v[x] |= self.v[y],
                0x2 => self.v[x] &= self.v[y],
                0x3 => self.v[x] ^= self.v[y],
                0x4 => {
                    let (sum, carry) = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = sum;
                    self.v[0xf] = carry as u8;
                }
                0x5 => {
                    let (diff, borrow) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = diff;
                    self.v[0xf] = !borrow as u8;
                }
                0x6 => {
                    let shift = self.v[y] >> 1;
                    let carry =  self.v[y] % 2;
                    self.v[x] = shift;
                    self.v[0xf] = carry;
                }
                0x7 => {
                    // y - x
                    let (diff, borrow) = self.v[y].overflowing_sub(self.v[x]);
                    self.v[x] = diff;
                    self.v[0xf] = !borrow as u8;
                }
                0xe => {
                    let shift = self.v[y] << 1;
                    let carry = if self.v[y] & 1 << 7 != 0 { 1 } else { 0 };
                    self.v[x] = shift;
                    self.v[0xf] = carry;
                }
                _ => err(),
            },
            0x9 => {
                assert_eq!(n, 0);
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            0xa => self.i = addr,
            0xd => self.draw_sprite(x, y, n),
            0xf => match k {
                0x1e => self.i += self.v[x] as u16,
                0x33 => {
                    let bcd = bcd_from_u8(self.v[x]);
                    for offset in 0..bcd.len() {
                        self.mem[self.i + offset as u16] = bcd[offset];
                    }
                }
                0x55 => {
                    // Write registers to memory.
                    for reg in 0..=x {
                        self.mem[self.i + reg as u16] = self.v[reg];
                    }
                }
                0x65 => {
                    // Read memory into registers.
                    for reg in 0..=x {
                        self.v[reg] = self.mem[self.i + reg as u16];
                    }
                }
                _ => err(),
            },
            _ => err(),
        }
    }

    fn draw_sprite(&mut self, x: u8, y: u8, n: u8) {
        assert!(x <= 0x0f);
        assert!(y <= 0x0f);
        assert!(n <= 0x0f);
        assert!(self.i + n as u16 <= Mem::LEN);
        let n = n as i8;

        let x = self.v[x] as i8;
        let y = self.v[y] as i8;
        self.v[0xf] = 0;

        for dy in 0..n {
            let sprite_row = self.mem[self.i + dy as u16];
            for dx in 0..8 {
                let pos = Point { x, y }.wrapping_add((dx, dy).into());

                let bit = 1 << (7 - dx);
                if sprite_row & bit != 0 {
                    match self.screen.flip(pos) {
                        Flip::Collision => self.v[0xf] = 1,
                        Flip::NoCollision => (),
                    }
                }
            }
        }
    }

    pub fn display(&self) {
        println!("{:?}", self.screen);
    }
}

/// Convert x to "big endian" binary coded decimal:
/// [hundreds, tens, ones]
fn bcd_from_u8(mut x: u8) -> [u8; 3] {
    // Start with [ones, tens, hundred], and then reverse.
    let mut digits = [0u8; 3];
    for i in 0..3 {
        digits[i] = x % 10;
        x /= 10;
    }

    digits.reverse();
    digits
}

/// Big endian byte (and bit) order.
fn nibbles_from_u16(x: u16) -> [u8; 4] {
    let a = (x & 0xf000) >> 4 * 3;
    let b = (x & 0x0f00) >> 4 * 2;
    let c = (x & 0x00f0) >> 4 * 1;
    let d = (x & 0x000f) >> 4 * 0;
    [a, b, c, d].map(|n| n as u8)
}
