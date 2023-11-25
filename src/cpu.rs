mod debug;
mod mem;
mod regs;
mod stack;

pub mod io;
pub mod screen;

use self::io::Chip8Io;
use mem::Mem;
use regs::Regs;
use screen::Point;
use stack::Stack;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Chip8<'a> {
    pc: u16,
    i: u16,
    stack: Stack,
    v: Regs,
    mem: Mem,
    io: &'a mut dyn Chip8Io,
}

impl<'a> Chip8<'a> {
    pub fn new(rom: &[u8], io: &'a mut dyn Chip8Io) -> Self {
        Self {
            pc: Mem::ROM_START,
            i: 0,
            stack: Stack::new(),
            v: Regs::new(),
            mem: Mem::new(rom),
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
            //eprintln!("{:#04x?}", self);

            self.io.update();
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
                0x00e0 => self.io.clear_screen(),
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
                0x1 => {
                    self.v[x] |= self.v[y];
                    self.v[0xf] = 0;
                }
                0x2 => {
                    self.v[x] &= self.v[y];
                    self.v[0xf] = 0;
                }
                0x3 => {
                    self.v[x] ^= self.v[y];
                    self.v[0xf] = 0;
                }
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
            0xc => self.v[x] = self.io.get_random_byte() & k,
            0xd => self.draw_sprite(x, y, n),
            0xe => match k {
                0x9e => {
                    if self.io.is_key_pressed(self.v[x]) {
                        self.pc += 2;
                    }
                }
                0xa1 => {
                    if !self.io.is_key_pressed(self.v[x]) {
                        self.pc += 2;
                    }
                }
                _ => err(),
            },
            0xf => match k {
                0x07 => self.v[x] = self.io.read_delay_timer(),
                0x0a => self.v[x] = self.io.blocking_get_key(),
                0x15 => self.io.write_delay_timer(self.v[x]),
                0x18 => self.io.write_sound_timer(self.v[x]),
                0x1e => self.i += self.v[x] as u16,
                0x29 => self.i = Mem::sprite_offset(self.v[x]),
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
                    self.i += x as u16 + 1;
                }
                0x65 => {
                    // Read memory into registers.
                    for reg in 0..=x {
                        self.v[reg] = self.mem[self.i + reg as u16];
                    }
                    self.i += x as u16 + 1;
                }
                _ => err(),
            },
            0x10.. => unreachable!(),
        }
    }

    fn draw_sprite(&mut self, x: u8, y: u8, n: u8) {
        assert!(x <= 0xf);
        assert!(y <= 0xf);
        assert!(n <= 0xf);
        assert!(self.i + n as u16 <= Mem::LEN);

        let xy = Point::from((self.v[x] as i8, self.v[y] as i8)).wrap();
        let sprite = &self.mem[self.i..self.i + n as u16];

        self.v[0xf] = self.io.draw_sprite(xy, sprite) as u8;
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
