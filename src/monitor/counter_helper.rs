use crate::model::counter::Counter;
use std::sync::Arc;
use tokio::sync::Mutex;

// TODO make use of async traits

pub async fn increment(counter: Arc<Mutex<Counter>>) {
    let mut p = counter.lock().await;
    p.data_count += 1;
}

pub async fn count(counter: Arc<Mutex<Counter>>) -> u64 {
    let p = counter.lock().await;
    p.data_count
}

pub async fn reset(counter: Arc<Mutex<Counter>>) {
    let mut p = counter.lock().await;
    p.data_count = 0;
}
