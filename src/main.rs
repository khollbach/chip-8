use anyhow::Result;
use chip_8::Chip8;
use std::io::{self, Read};

fn main() -> Result<()> {
    let mut rom = vec![];
    io::stdin().read_to_end(&mut rom)?;

    let mut emu = Chip8::new(&rom);
    for i in 0..100 {
        emu.step();
        eprintln!();
        eprintln!("i={i} {emu:#04x?}");
    }

    Ok(())
}
