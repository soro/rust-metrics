extern crate num;

use metric::{Metric, MetricValue};

pub trait Counter : Send + Sync {
    fn clear(&mut self);

    fn dec(&mut self, value: isize);

    fn inc(&mut self, value: isize);

    fn snapshot(&self) -> isize;
}

impl<T: Counter + Send + Sync> Metric for T {
    fn export_metric(&self) -> MetricValue {
        MetricValue::Counter(self.snapshot())
    }
}
