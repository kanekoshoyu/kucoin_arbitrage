use std::collections::HashMap;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

lazy_static::lazy_static! {
    static ref TIMERS: Mutex<HashMap<String, Instant>> = Mutex::new(HashMap::new());
}

/// Start
pub async fn start(name: String) {
    let mut timer = TIMERS.lock().await;
    timer.insert(name, Instant::now());
}

/// Stop
pub async fn stop(name: String) -> Result<Duration, String> {
    let now = Instant::now();
    let timer = TIMERS.lock().await;
    let stat = timer
        .get(&name)
        .ok_or(format!("global timer [{name:?}] is not found"))?;
    Ok(now.duration_since(*stat))
}
