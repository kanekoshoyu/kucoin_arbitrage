use crate::globals::{config, performance};
use kucoin_rs::tokio::time::{sleep, Duration};
use log::*;

fn report_status() -> Result<(), kucoin_rs::failure::Error> {
    info!("reporting");
    let data_rate = performance::data_count() / config::CONFIG.monitor_interval_sec;
    info!("Data rate: {data_rate:?} points/sec");
    // clear the data
    performance::reset();
    Ok(())
}

pub async fn background_routine() -> Result<(), kucoin_rs::failure::Error> {
    let monitor_delay = Duration::from_secs(config::CONFIG.monitor_interval_sec);
    loop {
        sleep(monitor_delay).await;
        report_status().expect("report status error");
    }
}
