#[derive(Debug, Clone)]
pub struct Stack {
    values: Vec<u16>,
}

impl Stack {
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    /// Panics on overflow.
    pub fn push(&mut self, value: u16) {
        if self.values.len() >= 16 {
            panic!("stack overflow");
        }
        self.values.push(value);
    }

    /// Panics on underflow.
    pub fn pop(&mut self) -> u16 {
        self.values.pop().unwrap()
    }
}
