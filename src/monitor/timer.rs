use eyre::Result;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

// static timers for global access
lazy_static::lazy_static! {
    static ref TIMERS: Mutex<HashMap<String, Instant>> = Mutex::new(HashMap::new());
}

/// start a global timer
pub async fn start(name: String) {
    let mut timer = TIMERS.lock().await;
    timer.insert(name, Instant::now());
}

/// stop/reads a global timer
pub async fn stop(name: String) -> Result<Duration> {
    let now = Instant::now();
    let timer = TIMERS.lock().await;
    let stat = timer.get(&name).ok_or(eyre::eyre!("stat was empty"))?;
    Ok(now.duration_since(*stat))
}
