extern crate num;

use metric::{Metric, MetricValue};

// Implementations have to offer safe interior mutability
pub trait Counter : Send + Sync {
    fn clear(&self);

    fn dec(&self, value: isize);

    fn inc(&self, value: isize);

    fn snapshot(&self) -> isize;
}

impl<T: Counter + Send + Sync> Metric for T {
    fn export_metric(&self) -> MetricValue {
        MetricValue::Counter(self.snapshot())
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::thread;
    use std::sync::Arc;

    fn spawn_incr<T: Counter + 'static>(sc: Arc<T>, n: isize) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            for _ in 0..n { sc.inc(1) }
        })
    }

    pub fn test_counter<T: Counter + 'static>(ctor: &Fn() -> T, iter_count: isize, thread_count: isize) -> Arc<T> {
        let c = Arc::new(ctor());

        let mut children = vec![];

        for _ in 0..thread_count {
            children.push(spawn_incr(c.clone(), iter_count));
        }

        for child in children {
            let res = child.join();
            assert!(res.is_ok());
        }

        c
    }
}
