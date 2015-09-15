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

    fn dec(&self, value: isize) { self.inc(-value) }

    fn snapshot(&self) -> isize { self.value.load(Ordering::Acquire) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use counters::Counter;
    use std::thread;
    use std::sync::Arc;

    fn spawn_incr(sc: Arc<AtomicCounter>, n: isize) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            for _ in 0..n { sc.inc(1) }
        })
    }

    #[test]
    fn test_atomic_counter() {
        let c = Arc::new(AtomicCounter::new());

        let thread_count = 16;
        let iter_count = 1000000;


         let mut children = vec![];

         for _ in 0..thread_count {
             children.push(spawn_incr(c.clone(), iter_count));
         }

         for child in children {
             let res = child.join();
             assert!(res.is_ok());
         }

        assert!(c.snapshot() as isize == thread_count * iter_count);

        c.clear();

        assert!(c.snapshot() as isize == 0);
    }
}
