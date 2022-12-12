extern crate kucoin_rs;

use kucoin_arbitrage::mirror::{Map, MIRROR};
use kucoin_rs::kucoin::{
    client::{Kucoin, KucoinEnv},
    model::market::OrderBookType,
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
    websocket::KucoinWebsocket,
};
use kucoin_rs::{futures::TryStreamExt, tokio};
use log::*;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), kucoin_rs::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    info!("Hello world");
    let credentials = kucoin_arbitrage::globals::config::credentials();
    info!("{credentials:#?}");
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;

    let res = api.get_orderbook("BTC-USDT", OrderBookType::Full).await;
    let res = res.unwrap();

    info!("{res:#?}");
    let orderbook = res.data.unwrap();
    // TODO: get all the order-book first
    unimplemented!();
    let url = api.get_socket_endpoint(WSType::Public).await?;
    let mut ws = api.websocket();

    // TODO: link the list_ticker to here and subscribe for all the tickers with BTC/USDT (Triangle)
    let subs = vec![WSTopic::OrderBook(vec!["ETH-BTC".to_string()])];
    ws.subscribe(url, subs).await?;

    info!("Async polling");
    // TODO: arbitrage performance analysis, such as arbitrage chance per minute

    let mirr = MIRROR.clone();
    tokio::spawn(async move { sync_tickers_rt(ws, mirr).await });
    kucoin_arbitrage::tasks::background_routine().await
}

use kucoin_arbitrage::strings::topic_to_symbol;

async fn sync_tickers_rt(
    mut ws: KucoinWebsocket,
    mirror: Arc<Mutex<Map>>,
) -> Result<(), kucoin_rs::failure::Error> {
    while let Some(msg) = ws.try_next().await? {
        // add matches for multi-subscribed sockets handling
        match msg {
            KucoinWebsocketMsg::OrderBookMsg(msg) => {
                kucoin_arbitrage::globals::performance::increment();
                order_message_received(msg, mirror.to_owned());
                // info!("{:#?}", msg);
            }
            KucoinWebsocketMsg::PongMsg(_msg) => {}
            KucoinWebsocketMsg::WelcomeMsg(_msg) => {}
            _ => {
                panic!("unexpected msgs received: {msg:?}")
            }
        }
    }
    Ok(())
}

use kucoin_rs::kucoin::model::websocket::{Level2, WSResp};

fn order_message_received(msg: WSResp<Level2>, mirror: Arc<Mutex<Map>>) {
    if msg.subject.ne("trade.l2update") {
        error!("unrecognised subject: {:?}", msg.subject);
        return;
    }
    // get the ticker name
    let ticker_name = topic_to_symbol(msg.topic).expect("wrong ticker format");
    // info!("Ticker received: {ticker_name}");
    let data = msg.data;
    // info!("{:#?}", data);
    let asks = data.changes.asks;
    let bids = data.changes.bids;
    for ask in asks.into_iter() {
        if ask.len().ne(&3) {
            panic!("wrong format");
        }
    }
    for bids in bids.into_iter() {
        if bids.len().ne(&3) {
            panic!("wrong format");
        }
    }
    // check if the ticker already exists in the map
    // let x = ticker_name.clone();
    // {
    //     let mut m = mirror.lock().unwrap();
    //     let tickers: &mut Map = &mut (*m);
    //     if let Some(data) = tickers.get_mut(&x) {
    //         // unimplemented!("found");
    //         data.symbol = msg.data;
    //     } else {
    //         tickers.insert(x, TickerInfo::new(msg.data));
    //     }
    // }
}
