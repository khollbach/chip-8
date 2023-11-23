mod debug;
mod mem;
mod regs;
mod screen;

use mem::Mem;
use regs::Regs;
use screen::Screen;

use crate::screen::{Flip, Point};

#[derive(Debug, Clone)]
pub struct Chip8 {
    pc: u16,
    sp: u8,
    i: u16,
    v: Regs,
    mem: Mem,
    screen: Screen,
}

impl Chip8 {
    pub fn new(rom: &[u8]) -> Self {
        Self {
            pc: Mem::ROM_START,
            sp: 0, // todo
            i: 0,
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
        self.pc += 2;

        let addr = instr & 0x0fff;
        let x = j & 0x0f;
        let y = (k & 0xf0) >> 4;
        let n = k & 0x0f;

        match instr {
            0x00e0 => self.screen = Screen::default(),
            0x6000..=0x6fff => self.v[x] = k,
            0xa000..=0xafff => self.i = addr,
            0xd000..=0xdfff => self.draw_sprite(x, y, n),
            0x1000..=0x1fff => self.pc = addr,
            _ => panic!(
                "unimplemented instruction: 0x{instr:04x?} (pc:0x{:04x?})",
                self.pc
            ),
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
