/*
    lazy statics: PERFORMANCE
*/
extern crate lazy_static;
use crate::model::performance::Performance;
use std::sync::Arc;
use tokio::sync::Mutex;

// Arc is the smart pointer used when we want to share the ownership across async functions
// Mutex is used when we want to ensure correctness of data with lock
// Arc has implicit 'static bound, so it cannot contain reference to local variable.
lazy_static::lazy_static! {
    static ref PERFORMANCE: Arc<Mutex<Performance>> =
        Arc::new(Mutex::new(Performance { data_count: 0 }));
}
pub async fn increment() {
    let mut p = PERFORMANCE.lock().await;
    p.data_count += 1;
}

pub async fn data_count() -> u64 {
    let p = PERFORMANCE.lock().await;
    p.data_count
}

pub async fn reset() {
    let mut p = PERFORMANCE.lock().await;
    p.data_count = 0;
}
