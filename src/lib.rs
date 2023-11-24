mod debug;
mod mem;
mod regs;
mod screen;
mod stack;

use std::fmt::{self, Debug};

use mem::Mem;
use regs::Regs;
use stack::Stack;

use crate::screen::{Flip, Point};

pub use screen::Screen;

#[derive(Debug)]
pub struct Chip8<'a> {
    pc: u16,
    i: u16,
    dt: u8,
    st: u8,
    stack: Stack,
    v: Regs,
    mem: Mem,
    screen: Screen,
    io: Chip8Io<'a>,
}

pub struct Chip8Io<'a> {
    pub render: Box<dyn FnMut(&Screen) + 'a>,
    /// Is the given key currently pressed? Keycodes are `0x0..=0xf`.
    pub is_key_pressed: Box<dyn FnMut(u8) -> bool + 'a>,
    /// Block until any key becomes pressed. Return that keycode.
    pub get_key: Box<dyn FnMut() -> u8 + 'a>,
    /// Return true if a timer "tick" occurred between the previous call to
    /// `poll_timer` and now.
    pub poll_timer: Box<dyn FnMut() -> bool + 'a>,
}

impl<'a> Chip8Io<'a> {
    pub fn new(
        render: impl FnMut(&Screen) + 'a,
        is_key_pressed: impl FnMut(u8) -> bool + 'a,
        get_key: impl FnMut() -> u8 + 'a,
        poll_timer: impl FnMut() -> bool + 'a,
    ) -> Self {
        Self {
            render: Box::new(render),
            is_key_pressed: Box::new(is_key_pressed),
            get_key: Box::new(get_key),
            poll_timer: Box::new(poll_timer),
        }
    }
}

impl<'a> Debug for Chip8Io<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Chip8Io").finish()
    }
}

impl<'a> Chip8<'a> {
    pub fn new(rom: &[u8], io: Chip8Io<'a>) -> Self {
        Self {
            pc: Mem::ROM_START,
            i: 0,
            dt: 0,
            st: 0,
            stack: Stack::new(),
            v: Regs::new(),
            mem: Mem::new(rom),
            screen: Screen::default(),
            io,
        }
    }

    pub fn run(mut self) {
        loop {
            // Detect "halt" instruction.
            // This is a hack to make testing easier.
            if self.would_halt() {
                break;
            }

            self.step();
            eprintln!("{:#04x?}", &self);
            if (self.io.poll_timer)() {
                self.dt = self.dt.saturating_sub(1);
                self.st = self.st.saturating_sub(1);
            }
        }
    }

    fn would_halt(&self) -> bool {
        let j = self.mem[self.pc];
        let k = self.mem[self.pc + 1];
        let instr = u16::from_be_bytes([j, k]);
        instr == 0x1000 | self.pc
    }

    fn step(&mut self) {
        debug_assert!(self.pc < Mem::LEN);
        // dbg!(self.pc);
        // debug_assert_eq!(self.pc % 2, 0);

        let j = self.mem[self.pc];
        let k = self.mem[self.pc + 1];
        let instr = u16::from_be_bytes([j, k]);
        let old_pc = self.pc;
        let err = || panic!("unimplemented: 0x{instr:04x} (pc=0x{old_pc:04x})");
        self.pc += 2;

        let [op, x, y, n] = nibbles_from_u16(instr);
        let addr = instr & 0x0fff;

        match op {
            0x0 => match instr {
                0x00e0 => {
                    self.screen = Screen::default();
                    (self.io.render)(&self.screen);
                }
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
                    let carry = self.v[y] % 2;
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
            0xb => self.pc = addr + self.v[0] as u16,
            0xc => self.v[x] = rand::random::<u8>() & k,
            0xd => {
                self.draw_sprite(x, y, n);
                (self.io.render)(&self.screen);
            }
            0xe => match k {
                0x9e => {
                    if (self.io.is_key_pressed)(self.v[x]) {
                        self.pc += 2;
                    }
                }
                0xa1 => {
                    if !(self.io.is_key_pressed)(self.v[x]) {
                        self.pc += 2;
                    }
                }
                _ => err(),
            },
            0xf => match k {
                0x07 => self.v[x] = self.dt,
                0x0a => self.v[x] = (self.io.get_key)(),
                0x15 => self.dt = self.v[x],
                0x18 => self.st = self.v[x],
                0x1e => self.i += self.v[x] as u16,
                0x29 => self.i = self.v[x] as u16 * 5,
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
            0x10.. => unreachable!(),
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
