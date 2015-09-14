use counters::Counter;

#[derive(Copy, Clone, Debug)]
pub struct SimpleCounter {
    pub value: isize
}

impl Counter for SimpleCounter {
    fn clear(&mut self) {
        self.value = 0;
    }

    fn dec(&mut self, value: isize) {
        self.value = self.value - value;
    }

    fn inc(&mut self, value: isize) {
        self.value = self.value + value;
    }

    fn snapshot(&self) -> isize {
        self.value
    }
}

impl SimpleCounter {
    pub fn new() -> SimpleCounter {
        SimpleCounter { value: 0 }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use counters::Counter;

    #[test]
    fn increment_by_1() {
        let mut c = SimpleCounter::new();
        c.inc(1);

        assert!(c.value == 1);
    }
}
