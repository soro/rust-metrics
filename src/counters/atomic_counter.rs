use std::sync::atomic::{Ordering, AtomicIsize};
use counters::Counter;

pub struct AtomicCounter {
    value: AtomicIsize
}

impl AtomicCounter {
    pub fn new() -> AtomicCounter {
        AtomicCounter { value: AtomicIsize::new(0) }
    }
}

impl Counter for AtomicCounter {
    fn clear(&self) { self.value.store(0, Ordering::SeqCst) }

    fn inc(&self, value: isize) { self.value.fetch_add(value, Ordering::Release); }

    fn dec(&self, value: isize) { self.value.fetch_sub(value, Ordering::Release); }

    fn snapshot(&self) -> isize { self.value.load(Ordering::Acquire) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use counters::Counter;
    use std::thread;
    use std::sync::Arc;
    use test::Bencher;
    use counters::counter::test_utils::*;

    fn ctor() -> AtomicCounter { AtomicCounter::new() }

    #[test]
    fn test_atomic_counter() {
        let thread_count = 16;
        let iter_count = 1000000;

        let c = test_counter(&ctor, iter_count, thread_count);

        assert!(c.snapshot() == thread_count * iter_count);

        c.clear();

        assert!(c.snapshot() == 0);
    }

    #[bench]
    fn bench_atomic_counter(b: &mut Bencher) {
        let thread_count = 16;
        let iter_count = 100000;

        b.iter(|| { test_counter(&ctor, iter_count, thread_count) })
    }
}
