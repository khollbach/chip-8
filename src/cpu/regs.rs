use std::{
    fmt::{self, Debug},
    ops::{Index, IndexMut},
};

use super::debug;

#[derive(Clone)]
pub struct Regs {
    regs: [u8; 16],
}

impl Regs {
    pub fn new() -> Self {
        Self { regs: [0; 16] }
    }
}

impl Index<u8> for Regs {
    type Output = u8;

    fn index(&self, index: u8) -> &Self::Output {
        &self.regs[index as usize]
    }
}

impl IndexMut<u8> for Regs {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.regs[index as usize]
    }
}

impl Debug for Regs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ ")?;
        debug::write_row(f, self.regs)?;
        write!(f, " ]")?;
        Ok(())
    }
}
