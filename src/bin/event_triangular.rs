use kucoin_arbitrage::broker::gatekeeper::kucoin::task_gatekeep_chances;
use kucoin_arbitrage::broker::order::kucoin::task_place_order;
use kucoin_arbitrage::broker::orderbook::kucoin::{task_pub_orderevent, task_sync_orderbook};
use kucoin_arbitrage::broker::strategy::all_taker_btc_usdt::task_pub_chance_all_taker_btc_usdt;
use kucoin_arbitrage::event::chance::ChanceEvent;
use kucoin_arbitrage::event::order::OrderEvent;
use kucoin_arbitrage::event::orderbook::OrderbookEvent;
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
    let api_clone = api.clone();
    let url = api.get_socket_endpoint(WSType::Public).await?;
    log::info!("Credentials setup");

    // Configure the pairs to subscribe and analyze
    let symbols = [
        "ETH-BTC".to_string(),
        "BTC-USDT".to_string(),
        "ETH-USDT".to_string(),
    ];

    // TODO get the coin configs
    //  get all the data from symbol first, then obtain the symbil info
    // let x = api.get_symbol_list(market);

    // Initialize the websocket
    let mut ws = api.websocket();
    let subs = vec![WSTopic::OrderBook(symbols.to_vec())];
    ws.subscribe(url, subs).await?;
    log::info!("Websocket subscription setup");

    // Create broadcast channels
    let (sender_orderbook, _) = channel::<OrderbookEvent>(256);
    let (mut sender_chance, _) = channel::<ChanceEvent>(128);
    let (mut sender_order, _) = channel::<OrderEvent>(64);

    log::info!("Channel setup");
    let mut rx_orderbook_1 = sender_orderbook.subscribe();
    let mut rx_orderbook_2 = sender_orderbook.subscribe();
    let mut rx_chance_2 = sender_chance.subscribe();
    let mut rx_order = sender_order.subscribe();

    let full_orderbook = Arc::new(Mutex::new(FullOrderbook::new()));
    let _res = tokio::join!(
        task_pub_orderevent(ws, sender_orderbook.clone()),
        task_sync_orderbook(&mut rx_orderbook_1, full_orderbook.clone()),
        task_pub_chance_all_taker_btc_usdt(
            &mut rx_orderbook_2,
            &mut sender_chance,
            full_orderbook.clone(),
        ),
        task_gatekeep_chances(&mut rx_chance_2, &mut sender_order),
        task_place_order(&mut rx_order, api_clone)
    );

    log::info!("tasks setup");

    // tokio::join!(task_sync_orderbook(&mut receiver, full_orderbook.clone()));
    // // tokio::spawn(async move { task_sync_orderbook(&mut receiver, full_orderbook.clone()).await });
    // log::info!("task_sync_orderbook setup");

    // let (mut sender_chance, _) = channel::<ChanceEvent>(128);

    // tokio::spawn(async move {
    //     task_pub_chance_all_taker_btc_usdt(
    //         &mut receiver_sync,
    //         &mut sender_chance,
    //         full_orderbook.clone(),
    //     )
    //     .await
    // });
    // log::info!("task_pub_chance_all_taker_btc_usdt setup");

    kucoin_arbitrage::tasks::background_routine().await
}
