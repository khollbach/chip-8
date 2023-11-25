use anyhow::Result;
use chip_8::TerminalIo;
use std::io::{self, Read};

fn main() -> Result<()> {
    let mut rom = vec![];
    io::stdin().read_to_end(&mut rom)?;

    chip_8::run(&rom, &mut TerminalIo::setup()?);

    Ok(())
}
