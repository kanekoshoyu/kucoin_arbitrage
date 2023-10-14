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
pub async fn stop(name: String) -> Result<Duration, failure::Error> {
    let now = Instant::now();
    let timer = TIMERS.lock().await;
    let stat = timer.get(&name).ok_or(failure::err_msg(format!(
        "global timer [{name:?}] is not found"
    )))?;
    Ok(now.duration_since(*stat))
}
