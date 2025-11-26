pub struct ChangeDetector<T> {
    last_value: Option<T>,
}

impl<T: PartialEq + Copy> ChangeDetector<T> {
    pub fn new() -> Self {
        ChangeDetector { last_value: None }
    }

    pub fn set(&mut self, value: T) -> (T, bool) {
        if let Some(last) = self.last_value {
            if last == value {
                return (last, false);
            }
        }
        self.last_value = Some(value);
        (value, true)
    }
}

#[derive(Default)]
pub struct SmoothingBuffer8 {
    buffer: [u32; 8],
    index: usize,
}

impl SmoothingBuffer8 {
    pub fn reset(&mut self, value: u32) {
        self.buffer = [value; 8];
    }

    pub fn push(&mut self, value: u32) {
        self.buffer[self.index] = value;
        self.index = (self.index + 1) % 8;
    }

    pub fn average(&self) -> u32 {
        let sum = self.buffer[self.index] * 6
            + self.buffer[self.index.wrapping_sub(1) % 8] * 6
            + self.buffer[self.index.wrapping_sub(2) % 8] * 5
            + self.buffer[self.index.wrapping_sub(3) % 8] * 5
            + self.buffer[self.index.wrapping_sub(4) % 8] * 4
            + self.buffer[self.index.wrapping_sub(5) % 8] * 3
            + self.buffer[self.index.wrapping_sub(6) % 8] * 2
            + self.buffer[self.index.wrapping_sub(7) % 8];
        sum >> 5 // divide by 32 (sum of the weights)
    }
}
