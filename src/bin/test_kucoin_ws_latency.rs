extern crate kucoin_rs;

use chrono::prelude::Local;
use kucoin_arbitrage::model::order::OrderSide;
use kucoin_arbitrage::translator::translator::OrderBookChangeTranslator;
use kucoin_rs::failure;
use kucoin_rs::futures::TryStreamExt;
use kucoin_rs::kucoin::{
    client::{Kucoin, KucoinEnv},
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
};

/// Test REST-to-WS Network Latency
/// place extreme order, receive extreme order, check time difference
#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Testing Kucoin REST-to-WS latency");
    let credentials = kucoin_arbitrage::globals::config::credentials();
    log::info!("{credentials:#?}");
    // Initialize the Kucoin API struct
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    let url = api.get_socket_endpoint(WSType::Public).await?;
    let mut ws = api.websocket();

    let subs = vec![WSTopic::OrderBook(vec!["BTC-USDT".to_string()])];
    // extreme order
    let test_symbol = "BTC-USDT";
    let test_price = 0.1;
    let test_volume = 0.1;

    let dt_order_placed = Local::now();
    // TODO set a valid limit order
    api.post_limit_order(
        0.to_string().as_str(),
        test_symbol,
        OrderSide::Buy.as_ref(),
        test_price.to_string().as_str(),
        test_volume.to_string().as_str(),
        None,
    )
    .await?;
    log::info!("Order placed {dt_order_placed}");
    ws.subscribe(url, subs).await?;

    log::info!("Async polling");
    let serial = 0;
    while let Some(msg) = ws.try_next().await? {
        match msg {
            KucoinWebsocketMsg::OrderBookMsg(msg) => {
                let (symbol, data) = msg.data.to_internal(serial);
                if let Some(volume) = data.ask.get(&ordered_float::OrderedFloat(test_price)) {
                    if volume.eq(&test_volume) && symbol.eq(test_symbol) {
                        let dt_order_reported = Local::now();
                        let delta = dt_order_reported - dt_order_placed;
                        log::info!("REST-to-WS: {}ms", delta.num_milliseconds());
                        return Ok(());
                    }
                }
            }
            KucoinWebsocketMsg::PongMsg(_) => continue,
            KucoinWebsocketMsg::WelcomeMsg(_) => continue,
            _ => {
                panic!("unexpected msgs received: {msg:?}")
            }
        }
    }
    Ok(())
}
