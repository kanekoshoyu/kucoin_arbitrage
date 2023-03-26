use kucoin_arbitrage::broker::orderbook::kucoin::{task_pub_orderevent, task_sync_orderbook};
use kucoin_arbitrage::broker::strategy::all_taker_btc_usdt::task_pub_chance_all_taker_btc_usdt;
use kucoin_arbitrage::event::chance::ChanceEvent;
use kucoin_arbitrage::model::orderbook::FullOrderbook;

use kucoin_rs::kucoin::{
    client::{Kucoin, KucoinEnv},
    model::websocket::{WSTopic, WSType},
};
use kucoin_rs::tokio;
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
    let (sender, mut receiver) = channel(256);
    let sender = Arc::new(sender);
    let mut receiver_sync = sender.subscribe();
    log::info!("Channel setup");

    // OrderEvent Task
    tokio::spawn(async move { task_pub_orderevent(ws, sender).await });
    log::info!("task_pub_orderevent setup");

    // Orderbook Sync Task
    let full_orderbook = Arc::new(Mutex::new(FullOrderbook::new()));
    let full_orderbook_arbitrage_clone = full_orderbook.clone();
    tokio::spawn(async move { task_sync_orderbook(&mut receiver, full_orderbook).await });
    log::info!("task_sync_orderbook setup");

    let (mut sender_chance, _) = channel::<ChanceEvent>(128);

    tokio::spawn(async move {
        task_pub_chance_all_taker_btc_usdt(
            &mut receiver_sync,
            &mut sender_chance,
            full_orderbook_arbitrage_clone,
        )
        .await
    });
    log::info!("task_pub_chance_all_taker_btc_usdt setup");

    kucoin_arbitrage::tasks::background_routine().await
}
