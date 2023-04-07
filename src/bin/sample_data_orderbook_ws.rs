extern crate kucoin_rs;
use kucoin_arbitrage::mirror::{Map, TickerInfo, MIRROR};
use kucoin_arbitrage::strings::topic_to_symbol;
use kucoin_rs::failure;
use kucoin_rs::futures::TryStreamExt;
use kucoin_rs::kucoin::{
    client::{Kucoin, KucoinEnv},
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
    websocket::KucoinWebsocket,
};
use kucoin_rs::tokio::{self};
use log::*;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    info!("Hello world");
    let credentials = kucoin_arbitrage::globals::config::credentials();
    info!("{credentials:#?}");
    // Initialize the Kucoin API struct
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    let url = api.get_socket_endpoint(WSType::Public).await?;
    let mut ws = api.websocket();

    // TODO: link the list_ticker to here and subscribe for all the tickers with BTC/USDT (Triangle)

    let subs = vec![WSTopic::OrderBook(vec!["ETH-BTC".to_string()])];
    ws.subscribe(url, subs).await?;

    info!("Async polling");
    // TODO: arbitrage performance analysis, such as arbitrage chance per minute

    let mirr = MIRROR.clone();
    tokio::spawn(async move { sync_tickers(ws, mirr).await });
    kucoin_arbitrage::tasks::background_routine().await
}

async fn sync_tickers(
    mut ws: KucoinWebsocket,
    mirror: Arc<Mutex<Map>>,
) -> Result<(), failure::Error> {
    while let Some(msg) = ws.try_next().await? {
        // add matches for multi-subscribed sockets handling
        match msg {
            KucoinWebsocketMsg::TickerMsg(msg) => {
                // info!("{:#?}", msg);
                if msg.subject.ne("trade.ticker") {
                    error!("unrecognised subject: {:?}", msg.subject);
                    continue;
                }
                // get the ticker name
                let ticker_name = topic_to_symbol(msg.topic).expect("wrong ticker format");
                info!("Ticker received: {ticker_name}");
                info!("{:?}", msg.data);

                // check if the ticker already exists in the map
                let x = ticker_name.clone();
                {
                    let mut m = mirror.lock().unwrap();
                    let tickers: &mut Map = &mut (*m);
                    if let Some(data) = tickers.get_mut(&x) {
                        // unimplemented!("found");
                        data.symbol = msg.data;
                    } else {
                        tickers.insert(x, TickerInfo::new(msg.data));
                    }
                }
                kucoin_arbitrage::globals::performance::increment();
            }
            KucoinWebsocketMsg::PongMsg(_) => {}
            KucoinWebsocketMsg::WelcomeMsg(_) => {}
            KucoinWebsocketMsg::OrderBookMsg(msg) => {
                let l2 = msg.data;
                info!("{l2:#?}")
            }
            _ => {
                panic!("unexpected msgs received: {msg:?}")
            }
        }
    }
    Ok(())
}
