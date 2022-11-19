pub struct AutoIncrement {
    value: i32,
}

impl AutoIncrement {
    pub fn new() -> Self {
        AutoIncrement { value: 0 }
    }

    pub fn take(&mut self) -> i32 {
        self.value += 1;

        self.value
    }
}
