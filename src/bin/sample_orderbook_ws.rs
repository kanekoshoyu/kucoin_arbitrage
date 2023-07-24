// Subscribes orderbook changes in WebSocket API
use kucoin_api::futures::TryStreamExt;
use kucoin_api::{
    client::{Kucoin, KucoinEnv},
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
    websocket::KucoinWebsocket,
};
use kucoin_arbitrage::broker::symbol::filter::symbol_with_quotes;
use kucoin_arbitrage::broker::symbol::kucoin::get_symbols;
use kucoin_arbitrage::model::counter::Counter;
use kucoin_arbitrage::model::symbol::SymbolInfo;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), kucoin_api::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Log setup");
    let counter = Arc::new(Mutex::new(Counter::new("api_input")));

    // credentials
    let credentials = kucoin_arbitrage::global::config::credentials();
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    let url = api.clone().get_socket_endpoint(WSType::Public).await?;
    log::info!("Credentials setup");

    // get all symbols concurrently
    let symbol_list = get_symbols(api.clone()).await;
    log::info!("Total exchange symbols: {:?}", symbol_list.len());

    // filter with either btc or usdt as quote
    let symbol_infos = symbol_with_quotes(&symbol_list, "BTC", "USDT");
    log::info!("Total symbols in scope: {:?}", symbol_infos.len());

    // change a list of SymbolInfo into a 2D list of WSTopic per session in max 100 index
    let subs = format_subscription_list(&symbol_infos);
    log::info!("Total orderbook WS sessions: {:?}", subs.len());

    // setup subscription and tasks per session
    for (i, sub) in subs.iter().enumerate() {
        let mut ws = api.websocket();
        ws.subscribe(url.clone(), sub.clone()).await?;
        tokio::spawn(sync_tickers(ws, counter.clone()));
        log::info!("{i:?}-th session of WS subscription setup");
    }
    let _res = tokio::join!(kucoin_arbitrage::global::task::background_routine(vec![
        counter.clone(),
    ]));
    panic!("Program should not arrive here")
}

async fn sync_tickers(
    mut ws: KucoinWebsocket,
    counter: Arc<Mutex<Counter>>,
) -> Result<(), kucoin_api::failure::Error> {
    while let Some(msg) = ws.try_next().await? {
        // add matches for multi-subscribed sockets handling
        match msg {
            KucoinWebsocketMsg::PongMsg(_) => {
                log::info!("Connection maintained")
            }
            KucoinWebsocketMsg::WelcomeMsg(_) => {
                log::info!("Connection setup")
            }
            KucoinWebsocketMsg::OrderBookMsg(msg) => {
                let _ = msg.data;
                kucoin_arbitrage::global::counter_helper::increment(counter.clone()).await;
            }
            _ => {
                panic!("unexpected msgs received: {msg:?}")
            }
        }
    }
    Ok(())
}

// TODO this bridges between API and the internal model, it should be placed in broker
fn format_subscription_list(infos: &[SymbolInfo]) -> Vec<Vec<WSTopic>> {
    // extract the names
    let symbols: Vec<String> = infos.iter().map(|info| info.symbol.clone()).collect();

    // setup 2D array of max length 100
    let max_sub_count = 100;
    let mut hundred_arrays: Vec<Vec<String>> = Vec::new();
    let mut hundred_array: Vec<String> = Vec::new();

    // feed into the 2D array
    for symbol in symbols {
        hundred_array.push(symbol);
        // 99 for the first one, because of the special BTC-USDT
        if hundred_arrays.is_empty() && hundred_array.len() == max_sub_count - 1 {
            hundred_arrays.push(hundred_array);
            hundred_array = Vec::new();
            continue;
        }
        // otherwise 100
        if hundred_array.len() == max_sub_count {
            hundred_arrays.push(hundred_array);
            hundred_array = Vec::new();
        }
    }

    // last array in current_subarray
    if !hundred_array.is_empty() {
        hundred_arrays.push(hundred_array);
    }

    let mut subs: Vec<Vec<WSTopic>> = Vec::new();
    let mut sub: Vec<WSTopic> = Vec::new();
    for sub_array in hundred_arrays {
        sub.push(WSTopic::OrderBook(sub_array));
        if sub.len() == 3 {
            subs.push(sub);
            sub = Vec::new();
        }
    }
    subs
}
