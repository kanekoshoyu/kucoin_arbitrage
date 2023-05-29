extern crate kucoin_api;
use kucoin_api::futures::TryStreamExt;
use kucoin_api::{
    client::{Kucoin, KucoinEnv},
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
    websocket::KucoinWebsocket,
};
use kucoin_arbitrage::broker::symbol::filter::symbol_with_quotes;
use kucoin_arbitrage::broker::symbol::kucoin::get_symbols;

#[tokio::main]
async fn main() -> Result<(), kucoin_api::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Log setup");

    // credentials
    let credentials = kucoin_arbitrage::global::config::credentials();
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    let url = api.clone().get_socket_endpoint(WSType::Public).await?;
    log::info!("Credentials setup");

    // get all symbols concurrently
    let symbol_list = get_symbols(api.clone()).await;
    log::info!("total exchange symbols: {:?}", symbol_list.len());

    // filter with either btc or usdt as quote
    let symbol_infos = symbol_with_quotes(&symbol_list, "BTC", "USDT");

    log::info!(
        "total symbols with both btc and usdt quote: {:?}",
        symbol_infos.len()
    );

    // extract the names
    let mut symbols = Vec::new();
    for symbol_info in symbol_infos.clone() {
        symbols.push(symbol_info.symbol);
    }

    // setup 2D array of max length 100
    let max_sub_count = 100;
    let mut divided_array: Vec<Vec<String>> = Vec::new();
    let mut current_subarray: Vec<String> = Vec::new();

    // feed into the 2D array
    for symbol in symbols {
        current_subarray.push(symbol);
        // 99 for the first one, because of the special BTC-USDT
        if divided_array.is_empty() && current_subarray.len() == max_sub_count - 1 {
            divided_array.push(current_subarray);
            current_subarray = Vec::new();
            continue;
        }
        // otherwise 100
        if current_subarray.len() == max_sub_count {
            divided_array.push(current_subarray);
            current_subarray = Vec::new();
        }
    }

    // last array in current_subarray
    if !current_subarray.is_empty() {
        divided_array.push(current_subarray);
    }

    let mut subs: Vec<Vec<WSTopic>> = Vec::new();
    let mut sub: Vec<WSTopic> = Vec::new();
    for sub_array in divided_array {
        sub.push(WSTopic::OrderBook(sub_array));
        if sub.len() == 3 {
            subs.push(sub);
            sub = Vec::new();
        }
    }

    log::info!("subs.len(): {:?}", subs.len());

    // Initialize the websocket
    let mut ws = api.websocket();
    let mut ws2 = api.websocket();
    ws.subscribe(url.clone(), subs[0].clone()).await?;
    ws2.subscribe(url.clone(), subs[1].clone()).await?;
    log::info!("Websocket subscription setup");

    tokio::spawn(async move { sync_tickers(ws).await });
    tokio::spawn(async move { sync_tickers(ws2).await });
    kucoin_arbitrage::global::task::background_routine().await
}

async fn sync_tickers(mut ws: KucoinWebsocket) -> Result<(), kucoin_api::failure::Error> {
    while let Some(msg) = ws.try_next().await? {
        // add matches for multi-subscribed sockets handling
        match msg {
            KucoinWebsocketMsg::PongMsg(_) => {
                log::info!("Connection maintained")
            },
            KucoinWebsocketMsg::WelcomeMsg(_) => {
                log::info!("Connection setup")
            },
            KucoinWebsocketMsg::OrderBookMsg(msg) => {
                let _ = msg.data;
                kucoin_arbitrage::global::performance::increment().await;
            }
            _ => {
                panic!("unexpected msgs received: {msg:?}")
            }
        }
    }
    Ok(())
}
