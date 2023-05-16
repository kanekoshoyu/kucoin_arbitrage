use kucoin_api::{
    client::{Kucoin, KucoinEnv},
    model::websocket::{WSTopic, WSType},
};
use kucoin_arbitrage::broker::orderbook::kucoin::{task_pub_orderbook_event, task_sync_orderbook};
use kucoin_arbitrage::model::orderbook::FullOrderbook;
use std::sync::Arc;
use tokio::sync::broadcast::channel;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), kucoin_api::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Log setup");

    // credentials
    let credentials = kucoin_arbitrage::global::config::credentials();
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
    tokio::spawn(async move { task_pub_orderbook_event(ws, sender).await });
    log::info!("task_pub_orderevent setup");

    // Orderbook Sync Task
    let full_orderbook = Arc::new(Mutex::new(FullOrderbook::new()));
    tokio::spawn(async move { task_sync_orderbook(receiver, sender_best, full_orderbook).await });
    log::info!("task_sync_orderbook setup");

    kucoin_arbitrage::global::task::background_routine().await
}
