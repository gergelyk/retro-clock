pub struct CountDown {
    initial: u32,
    value: u32,
}

impl CountDown {
    pub fn new(initial: u32) -> Self {
        Self { initial, value: 0 }
    }

    pub fn reset(&mut self) {
        self.value = self.initial;
    }

    pub fn decrement(&mut self) -> bool {
        let mut is_last_one = false;
        if self.value > 0 {
            self.value -= 1;
            if self.value == 0 {
                is_last_one = true;
            }
        }
        is_last_one
    }
}
