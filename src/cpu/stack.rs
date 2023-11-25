#[derive(Debug, Clone)]
pub struct Stack {
    values: Vec<u16>,
}

const CAPACITY: usize = 16;

impl Stack {
    pub fn new() -> Self {
        Self {
            values: Vec::with_capacity(CAPACITY),
        }
    }

    /// Panics on overflow.
    pub fn push(&mut self, value: u16) {
        if self.values.len() >= CAPACITY {
            panic!("stack overflow");
        }
        self.values.push(value);
    }

    /// Panics on underflow.
    pub fn pop(&mut self) -> u16 {
        self.values.pop().unwrap()
    }
}
