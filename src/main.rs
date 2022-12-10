extern crate kucoin_rs;

use kucoin_rs::failure;
use kucoin_rs::futures::TryStreamExt;
use kucoin_rs::kucoin::{
    client::{Kucoin, KucoinEnv},
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
    websocket::KucoinWebsocket,
};
use kucoin_rs::tokio::{self};
use log::*;

// Arc has implicit 'static bound, so it cannot contain reference to local variable.
#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    info!("Hello world");
    let credentials = kucoin_arbitrage::globals::config::credentials();
    info!("{credentials:#?}");
    // Initialize the Kucoin API struct
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    // Generate the dynamic Public or Private websocket url and endpoint from Kucoin
    // which includes a token required for connecting
    let url = api.get_socket_endpoint(WSType::Public).await?;
    // Initialize the websocket
    let mut ws = api.websocket();

    // Generate a Vec<WSTopic> of desired subs.
    // Note they need to be public or private depending on the url

    // TODO: link the list_ticker to here and subscribe for all the tickers with BTC/USDT (Triangle)
    let subs = vec![WSTopic::Ticker(vec![
        "ETH-BTC".to_string(),
        "BTC-USDT".to_string(),
        "ETH-USDT".to_string(),
    ])];
    ws.subscribe(url, subs).await?;
    info!("Async polling");
    tokio::spawn(async move { poll_task(ws).await });
    kucoin_arbitrage::tasks::background_routine().await
}

// TODO; store the data into a map that mirrors a ticker status
async fn poll_task(mut ws: KucoinWebsocket) -> Result<(), failure::Error> {
    while let Some(msg) = ws.try_next().await? {
        match msg {
            KucoinWebsocketMsg::TickerMsg(_msg) => {
                kucoin_arbitrage::globals::performance::increment();
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
