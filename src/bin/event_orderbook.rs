use kucoin_arbitrage::broker::orderbook::kucoin::{task_pub_orderevent, task_sync_orderbook};
use kucoin_arbitrage::model::orderbook::FullOrderbook;
use kucoin_rs::kucoin::{
    client::{Kucoin, KucoinEnv},
    model::websocket::{WSTopic, WSType},
};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast::channel;

#[tokio::main]
async fn main() -> Result<(), kucoin_rs::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Log setup");

    // credentials
    let credentials = kucoin_arbitrage::globals::config::credentials();
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    let url = api.get_socket_endpoint(WSType::Public).await?;
    log::info!("Credentials setup");

    // Initialize the websocket
    let mut ws = api.websocket();
    let subs = vec![
        WSTopic::OrderBook(vec![
            "ETH-BTC".to_string(),
            "BTC-USDT".to_string(),
            "ETH-USDT".to_string(),
        ]),
        // WSTopic::OrderBookChange(vec!["ETH-BTC".to_string(), "BTC-USDT".to_string()]),
    ];
    ws.subscribe(url, subs).await?;
    log::info!("Websocket subscription setup");

    // Create a broadcast channel.
    let (sender, receiver) = channel(256);
    let (sender_best, _) = channel(64);
    log::info!("Channel setup");

    // OrderEvent Task
    tokio::spawn(async move { task_pub_orderevent(ws, sender).await });
    log::info!("task_pub_orderevent setup");

    // Orderbook Sync Task
    let full_orderbook = Arc::new(Mutex::new(FullOrderbook::new()));
    tokio::spawn(async move { task_sync_orderbook(receiver, sender_best, full_orderbook).await });
    log::info!("task_sync_orderbook setup");

    kucoin_arbitrage::tasks::background_routine().await
}
