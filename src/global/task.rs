use crate::global::{config, counter_helper};
use crate::model::counter::Counter;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

async fn report_status(
    counters: Vec<Arc<Mutex<Counter>>>,
) -> Result<(), kucoin_api::failure::Error> {
    log::info!("Reporting");
    for counter in counters.iter() {
        let data_rate =
            counter_helper::count(counter.clone()).await / config::CONFIG.monitor_interval_sec;
        log::info!("Data rate: {data_rate:?} points/sec");
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
