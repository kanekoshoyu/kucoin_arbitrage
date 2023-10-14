use std::sync::Arc;
use tokio::sync::Mutex;

/// Counter for system monitor
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Counter {
    pub name: &'static str,
    pub data_count: u64,
}

impl Counter {
    // Constructs a new instance of [`Counter`].
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            data_count: 0,
        }
    }
}

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
