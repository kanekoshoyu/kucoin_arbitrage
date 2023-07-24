/// Test WebSocket Message Rate
/// Subscribe to messages, run for 10 seconds, get the rate respectively
use kucoin_api::failure;
use kucoin_api::futures::TryStreamExt;
use kucoin_api::{
    client::{Kucoin, KucoinEnv},
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
    websocket::KucoinWebsocket,
};
use kucoin_arbitrage::global::counter_helper;
use kucoin_arbitrage::model::counter::Counter;
use std::sync::Arc;
use tokio::sync::Mutex;

/// main function
#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    let counter = Arc::new(Mutex::new(Counter::new("api_input")));
    log::info!("Testing Kucoin WS Message Rate");
    let credentials = kucoin_arbitrage::global::config::credentials();
    log::info!("{credentials:#?}");
    // Initialize the Kucoin API struct
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    let url = api.get_socket_endpoint(WSType::Public).await?;
    let mut ws = api.websocket();
    let symbols = [
        "ETH-BTC".to_string(),
        "BTC-USDT".to_string(),
        "ETH-USDT".to_string(),
    ];
    let subs = vec![WSTopic::OrderBook(symbols.to_vec())];
    ws.subscribe(url, subs).await?;

    log::info!("Async polling");
    tokio::spawn(sync_tickers(ws, counter.clone()));
    let _res = tokio::join!(kucoin_arbitrage::global::task::background_routine(vec![
        counter.clone()
    ]));
    panic!("Program should not arrive here")
}

async fn sync_tickers(
    mut ws: KucoinWebsocket,
    counter: Arc<Mutex<Counter>>,
) -> Result<(), failure::Error> {
    while let Some(msg) = ws.try_next().await? {
        match msg {
            KucoinWebsocketMsg::OrderBookMsg(_msg) => {
                // TODO make counter more generic
                counter_helper::reset(counter.clone()).await;
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
