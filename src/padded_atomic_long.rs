use std::sync::atomic::AtomicIsize;

#[allow(dead_code)]
pub struct PaddedAtomicLong {
    pub value: AtomicIsize,
    padding: [usize; 8]
}

#[allow(dead_code)]
impl PaddedAtomicLong {
    pub fn new() -> PaddedAtomicLong {
        PaddedAtomicLong { value: AtomicIsize::new(0), padding: [0; 8] }
    }
}
