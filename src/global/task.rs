use crate::global::{config, counter_helper};
use crate::model::counter::Counter;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

async fn report_status(
    counters: Vec<Arc<Mutex<Counter>>>,
) -> Result<(), kucoin_api::failure::Error> {
    log::info!("Reporting broadcast data rate");
    for counter in counters.iter() {
        let (name, count) = {
            let p = counter.lock().await;
            (p.name, p.data_count)
        };
        log::info!(
            "{name:?}: {count:?} points ({:?}pps)",
            count / config::CONFIG.monitor_interval_sec
        );
        // clear the data
        counter_helper::reset(counter.clone()).await;
    }
    Ok(())
}

pub async fn background_routine(
    counters: Vec<Arc<Mutex<Counter>>>,
) -> Result<(), kucoin_api::failure::Error> {
    let monitor_delay = Duration::from_secs(config::CONFIG.monitor_interval_sec);
    loop {
        sleep(monitor_delay).await;
        report_status(counters.clone())
            .await
            .expect("report status error");
    }
}
