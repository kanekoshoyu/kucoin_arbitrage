extern crate lazy_static;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone, Copy)]
pub struct Performance {
    pub data_count: u64,
}

// Arc is the smart pointer used when we want to share the ownership across async functions
// Mutex is used when we want to ensure correctness of data with lock
// Arc has implicit 'static bound, so it cannot contain reference to local variable.
lazy_static::lazy_static! {
    static ref PERFORMANCE: Arc<Mutex<Performance>> =
        Arc::new(Mutex::new(Performance { data_count: 0 }));
}
pub fn increment() {
    let mut p = PERFORMANCE.lock().unwrap();
    (*p).data_count = (*p).data_count + 1;
}

pub fn data_count() -> u64 {
    let p = PERFORMANCE.lock().unwrap();
    return (*p).data_count;
}

pub fn reset() {
    let mut p = PERFORMANCE.lock().unwrap();
    (*p).data_count = 0;
}
