use crate::global::{config, performance};
use tokio::time::{sleep, Duration};

async fn report_status() -> Result<(), kucoin_api::failure::Error> {
    log::info!("reporting");
    let data_rate = performance::data_count().await / config::CONFIG.monitor_interval_sec;
    log::info!("Data rate: {data_rate:?} points/sec");
    // clear the data
    performance::reset().await;
    Ok(())
}

pub async fn background_routine() -> Result<(), kucoin_api::failure::Error> {
    let monitor_delay = Duration::from_secs(config::CONFIG.monitor_interval_sec);
    loop {
        sleep(monitor_delay).await;
        report_status().await.expect("report status error");
    }
}
