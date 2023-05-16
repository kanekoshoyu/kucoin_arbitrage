extern crate kucoin_api;
use kucoin_api::failure;
use kucoin_api::futures::TryStreamExt;
use kucoin_api::{
    client::{Kucoin, KucoinEnv},
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
    websocket::KucoinWebsocket,
};
use kucoin_arbitrage::strings::topic_to_symbol;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Hello world");
    let credentials = kucoin_arbitrage::global::config::credentials();
    log::info!("{credentials:#?}");
    // Initialize the Kucoin API struct
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    let url = api.get_socket_endpoint(WSType::Public).await?;
    let mut ws = api.websocket();

    // TODO: link the list_ticker to here and subscribe for all the tickers with BTC/USDT (Triangle)

    let subs = vec![WSTopic::OrderBook(vec!["ETH-BTC".to_string()])];
    ws.subscribe(url, subs).await?;

    log::info!("Async polling");
    // TODO: arbitrage performance analysis, such as arbitrage chance per minute

    tokio::spawn(async move { sync_tickers(ws).await });
    kucoin_arbitrage::global::task::background_routine().await
}

async fn sync_tickers(mut ws: KucoinWebsocket) -> Result<(), failure::Error> {
    while let Some(msg) = ws.try_next().await? {
        // add matches for multi-subscribed sockets handling
        match msg {
            KucoinWebsocketMsg::TickerMsg(msg) => {
                // info!("{:#?}", msg);
                if msg.subject.ne("trade.ticker") {
                    log::error!("unrecognised subject: {:?}", msg.subject);
                    continue;
                }
                // get the ticker name
                let ticker_name = topic_to_symbol(msg.topic).expect("wrong ticker format");
                log::info!("Ticker received: {ticker_name}");
                log::info!("{:?}", msg.data);
                kucoin_arbitrage::global::performance::increment().await;
            }
            KucoinWebsocketMsg::PongMsg(_) => continue,
            KucoinWebsocketMsg::WelcomeMsg(_) => continue,
            KucoinWebsocketMsg::OrderBookMsg(msg) => {
                let l2 = msg.data;
                log::info!("{l2:#?}")
            }
            _ => {
                panic!("unexpected msgs received: {msg:?}")
            }
        }
    }
    Ok(())
}
