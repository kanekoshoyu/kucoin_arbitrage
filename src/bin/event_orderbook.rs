/// Syncs orderbook
use kucoin_api::client::{Kucoin, KucoinEnv};
use kucoin_api::model::websocket::WSTopic;
use kucoin_arbitrage::broker::orderbook::kucoin::{task_pub_orderbook_event, task_sync_orderbook};
use kucoin_arbitrage::model::counter::Counter;
use kucoin_arbitrage::model::orderbook::FullOrderbook;
use std::sync::Arc;
use tokio::sync::broadcast::channel;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Log setup");
    let counter = Arc::new(Mutex::new(Counter::new("api_input")));

    // config
    let config = kucoin_arbitrage::config::from_file("config.toml")?;
    let monitor_interval = config.behaviour.monitor_interval_sec;
    let api = Kucoin::new(KucoinEnv::Live, Some(config.kucoin_credentials()))?;
    log::info!("Credentials setup");

    let topics = vec![WSTopic::OrderBook(vec![
        "ETH-BTC".to_string(),
        "BTC-USDT".to_string(),
        "ETH-USDT".to_string(),
    ])];

    // Create a broadcast channel.
    let (sender, receiver) = channel(256);
    let (sender_best, _) = channel(64);
    log::info!("Channel setup");

    // OrderEvent Task
    tokio::spawn(task_pub_orderbook_event(api.clone(), topics, sender));
    log::info!("task_pub_orderevent setup");

    // Orderbook Sync Task
    let full_orderbook = Arc::new(Mutex::new(FullOrderbook::new()));
    tokio::spawn(task_sync_orderbook(
        receiver,
        sender_best,
        full_orderbook,
        counter.clone(),
    ));
    log::info!("task_sync_orderbook setup");

    let _ = tokio::join!(kucoin_arbitrage::global::task::task_log_mps(
        vec![counter.clone()],
        monitor_interval as u64
    ));
    panic!("Program should not arrive here")
}
