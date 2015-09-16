use std::ptr::{self, write_bytes};
use std::sync::atomic::Ordering::{Acquire, Release};
use std::rt::heap::{allocate, deallocate};
use std::cell::UnsafeCell;
use std::mem;

use counters::Counter;
use padded_atomic_long::PaddedAtomicLong;
use thread_hash::ThreadHash;

#[allow(dead_code)]
thread_local!(static PROBE: UnsafeCell<ThreadHash> = UnsafeCell::new(ThreadHash::new()));

struct ShardedAtomicCounter {
    log_size: usize,
    cells: ptr::Unique<PaddedAtomicLong>
}

#[allow(dead_code)]
impl ShardedAtomicCounter {
    fn calc_mem_size(log_size: usize) -> usize { (1 << log_size) * mem::size_of::<PaddedAtomicLong>() }

    unsafe fn alloc_underlying(log_size: usize) -> *mut PaddedAtomicLong {
        let mem_size = ShardedAtomicCounter::calc_mem_size(log_size);
        let cell_array = allocate(mem_size, mem::align_of::<PaddedAtomicLong>());
        write_bytes(cell_array, 0, mem_size);
        mem::transmute(cell_array)
    }

    unsafe fn dealloc_underlying(log_size: usize, base_ptr: *const PaddedAtomicLong) {
        let mem_size = ShardedAtomicCounter::calc_mem_size(log_size);
        deallocate(base_ptr as *mut u8, mem_size, mem::align_of::<PaddedAtomicLong>())
    }

    unsafe fn index_ptr(base_ptr: *mut PaddedAtomicLong, log_size: usize, n: isize) -> *mut PaddedAtomicLong {
        base_ptr.offset(n & ((1 << log_size) - 1))
    }

    #[inline(always)]
    fn sub_op(cell: &PaddedAtomicLong, val: isize) -> isize {
        cell.value.fetch_sub(val, Release)
    }

    #[inline(always)]
    fn add_op(cell: &PaddedAtomicLong, val: isize) -> isize {
        cell.value.fetch_add(val, Release)
    }

    #[inline(always)]
    unsafe fn op_and_balance(&self, val: isize, op: fn(&PaddedAtomicLong, isize) -> isize) {
        PROBE.with(|thref| {
            let th = thref.get();
            let probe = (*th).value;
            let raw_cell_ptr = ShardedAtomicCounter::index_ptr(*self.cells as *mut PaddedAtomicLong, self.log_size, probe);
            let cell = &*raw_cell_ptr;
            let seen_before = cell.value.load(Acquire);
            let seen_at_xchg = op(cell, val);
            if seen_before != seen_at_xchg {
                *th = ThreadHash::rehash(probe);
            }
        })
    }

    pub fn new(log_size: usize) -> ShardedAtomicCounter {
        assert!(log_size > 0, "trying to create long adder with table of log(size) <= 0");
        unsafe {
            ShardedAtomicCounter {
                log_size: log_size,
                cells: ptr::Unique::new(ShardedAtomicCounter::alloc_underlying(log_size))
            }
        }
    }

    pub fn add(&self, val: isize) {
        unsafe { self.op_and_balance(val, ShardedAtomicCounter::add_op) };
    }

    pub fn sub(&self, val: isize) {
        unsafe { self.op_and_balance(val, ShardedAtomicCounter::sub_op) };
    }

    // This is NOT guaranteed to yield the real current value
    pub fn snapshot(&self) -> isize {
        let mut acc = 0;
        let base_ptr = *self.cells as *mut PaddedAtomicLong;
        for i in 0..(1 << self.log_size) {
            acc += unsafe { (&*base_ptr.offset(i)).value.load(Acquire) };
        }
        acc
    }

    // This is NOT thread safe and does not guarantee a complete reset
    pub fn clear(&self) {
        let base_ptr = *self.cells as *mut PaddedAtomicLong;
        for i in 0..(1 << self.log_size) {
            unsafe { (&*base_ptr.offset(i)).value.store(0, Release) };
        }
    }
}

impl Drop for ShardedAtomicCounter {
    fn drop(&mut self) { unsafe {
        ShardedAtomicCounter::dealloc_underlying(
            self.log_size,
            self.cells.get()
        );
    }}
}

impl Counter for ShardedAtomicCounter {
    fn clear(&self) {
        self.clear()
    }

    fn dec(&self, value: isize) {
        self.sub(value)
    }

    fn inc(&self, value: isize) {
        self.add(value)
    }

    fn snapshot(&self) -> isize {
        self.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use counters::Counter;
    use super::ShardedAtomicCounter;
    use std::thread;
    use std::sync::Arc;
    use test::Bencher;
    use counters::counter::test_utils::*;

    fn ctor() -> ShardedAtomicCounter { ShardedAtomicCounter::new(4) }

    #[test]
    fn test_sharded_counter() {

        let thread_count = 16;
        let iter_count = 1000000;

        let c = test_counter(&ctor, iter_count, thread_count);

        assert!(c.snapshot() as isize == thread_count * iter_count);

        c.clear();

        assert!(c.snapshot() as isize == 0);
    }

    #[bench]
    fn bench_sharded_counter(b: &mut Bencher) {

        let thread_count = 16;
        let iter_count = 100000;
        
        b.iter(|| { test_counter(&ctor, iter_count, thread_count) })
    }
}
