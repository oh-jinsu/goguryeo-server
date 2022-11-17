pub struct AutoIncrement {
    value: usize,
}

impl AutoIncrement {
    pub fn new() -> Self {
        AutoIncrement { value: 0 }
    }

    pub fn take(&mut self) -> usize {
        self.value += 1;

        self.value
    }
}
