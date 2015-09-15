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
