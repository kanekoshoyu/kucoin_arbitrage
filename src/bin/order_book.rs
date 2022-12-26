extern crate kucoin_rs;

use kucoin_arbitrage::globals::{orderbook, symbol};
use kucoin_arbitrage::{strings, tickers};
use kucoin_rs::futures::future;
use kucoin_rs::kucoin::{
    client::{Kucoin, KucoinEnv},
    model::market::OrderBookType,
    model::websocket::{KucoinWebsocketMsg, Level2, WSResp, WSTopic, WSType},
    websocket::KucoinWebsocket,
};
use kucoin_rs::{futures::TryStreamExt, tokio};
use log::*;

async fn insert_single_orderbook_from_api(
    api: Kucoin,
    symbol_name: String,
) -> Result<(), kucoin_rs::failure::Error> {
    let ob_type = OrderBookType::L20;
    let res = api.get_orderbook(symbol_name.as_str(), ob_type).await?;
    if res.data.is_none() {
        panic!("OrderBook error: {:#?}", res.msg.unwrap());
    }
    // store globally
    orderbook::insert_book(symbol_name, res.data.unwrap());
    Ok(())
}

// load all the symbols in batch with join_all(), which saves lots of time
async fn insert_all_orderbooks_from_api(
    api: Kucoin,
    list: Vec<String>,
) -> Result<(), kucoin_rs::failure::Error> {
    let futures: Vec<_> = list
        .iter()
        .map(|name| insert_single_orderbook_from_api(api.clone(), name.to_owned()))
        .collect();
    future::join_all(futures).await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), kucoin_rs::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    info!("Hello world");
    let credentials = kucoin_arbitrage::globals::config::credentials();
    info!("{credentials:#?}");
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    // generate symbol whitelist
    let quote1 = "BTC";
    let quote2 = "USDT";
    let whitelist = tickers::symbol_whitelisted(api.clone(), quote1, quote2).await;
    let whitelist = whitelist.unwrap();
    let symbolmap = tickers::symbol_list_filtered(api.clone(), whitelist.clone()).await;
    let symbolmap = symbolmap.unwrap();
    // store globally
    symbol::insert_symbolmap(symbolmap).unwrap();

    // generate orderbook
    info!("Generating orderbook, {} in total", whitelist.len());
    let _res = insert_all_orderbooks_from_api(api.clone(), whitelist.clone()).await;
    info!("Generated");

    // let orderbook = res.data.unwrap();
    // TODO: get all the order-book first

    let url = api.get_socket_endpoint(WSType::Public).await?;
    let mut ws = api.websocket();

    // TODO: link the list_ticker to here and subscribe for all the tickers with BTC/USDT (Triangle)
    // let subs = vec![WSTopic::OrderBook(vec!["ETH-BTC".to_string()])];
    // TODO: somehow whitelist cannot be passed to the subscription directly, require some process here
    let v = vec![
        whitelist.get(0).unwrap().to_owned(),
        whitelist.get(1).unwrap().to_owned(),
        whitelist.get(2).unwrap().to_owned(),
    ];
    let subs = vec![WSTopic::OrderBook(v)];
    ws.subscribe(url, subs).await?;

    info!("Async polling");
    // TODO: arbitrage performance analysis, such as arbitrage chance per minute

    tokio::spawn(async move { sync_tickers_rt(ws).await });
    kucoin_arbitrage::tasks::background_routine().await
}

async fn sync_tickers_rt(mut ws: KucoinWebsocket) -> Result<(), kucoin_rs::failure::Error> {
    while let Some(msg) = ws.try_next().await? {
        // add matches for multi-subscribed sockets handling
        match msg {
            KucoinWebsocketMsg::OrderBookMsg(msg) => {
                kucoin_arbitrage::globals::performance::increment();
                order_message_received(msg);
                // info!("MESSAGE: {msg:#?}");
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

fn order_message_received(msg: WSResp<Level2>) {
    if msg.subject.ne("trade.l2update") {
        error!("unrecognised subject: {:?}", msg.subject);
        return;
    }
    // info!("received");
    // get the ticker name
    let ticker_name = strings::topic_to_symbol(msg.topic).expect("wrong ticker format");
    // info!("Ticker received: {ticker_name}");
    let data = msg.data;
    // info!("{:#?}", data);
    orderbook::update_ws(ticker_name, data).expect("failed storing changes locally");
}
