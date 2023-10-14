use crate::monitor::counter::{self, Counter};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

/// log counters
async fn log_mps(
    counters: Vec<Arc<Mutex<Counter>>>,
    interval: u64,
) -> Result<(), kucoin_api::failure::Error> {
    log::info!("Broadcast channel data rate");
    for counter in counters.iter() {
        let (name, count) = {
            let p = counter.lock().await;
            (p.name, p.data_count)
        };
        log::info!("{name:10}: {count:5} points ({:5}mps)", count / interval);
        // clear the data
        counter::reset(counter.clone()).await;
    }
    Ok(())
}
/// log counters as a task
pub async fn task_log_mps(
    counters: Vec<Arc<Mutex<Counter>>>,
    interval: u64,
) -> Result<(), kucoin_api::failure::Error> {
    let monitor_delay = Duration::from_secs(interval);
    loop {
        sleep(monitor_delay).await;
        log_mps(counters.clone(), interval)
            .await
            .expect("report status error");
    }
}
