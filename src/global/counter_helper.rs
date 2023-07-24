use crate::model::counter::Counter;
use std::sync::Arc;
use tokio::sync::Mutex;

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
