use kucoin_api::{
    client::{Kucoin, KucoinEnv},
    model::market::OrderBookType,
    model::websocket::{WSTopic, WSType},
};
use kucoin_arbitrage::broker::gatekeeper::kucoin::task_gatekeep_chances;
use kucoin_arbitrage::broker::order::kucoin::task_place_order;
use kucoin_arbitrage::broker::orderbook::kucoin::{task_pub_orderbook_event, task_sync_orderbook};
use kucoin_arbitrage::broker::symbol::filter::{symbol_with_quotes, vector_to_hash};
use kucoin_arbitrage::broker::symbol::kucoin::get_symbols;
use kucoin_arbitrage::event::chance::ChanceEvent;
use kucoin_arbitrage::event::order::OrderEvent;
use kucoin_arbitrage::event::orderbook::OrderbookEvent;
use kucoin_arbitrage::model::orderbook::FullOrderbook;
use kucoin_arbitrage::model::symbol::SymbolInfo;
use kucoin_arbitrage::strategy::all_taker_btc_usd::task_pub_chance_all_taker_btc_usd;
use kucoin_arbitrage::translator::traits::OrderBookTranslator;
use std::sync::Arc;
use tokio::sync::broadcast::channel;
use tokio::sync::Mutex;

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
    log::info!("Total exchange symbols: {:?}", symbol_list.len());

    // filter with either btc or usdt as quote
    let symbol_infos = symbol_with_quotes(&symbol_list, "BTC", "USDT");
    let hash_symbols = Arc::new(Mutex::new(vector_to_hash(&symbol_infos)));
    log::info!("Total symbols in scope: {:?}", symbol_infos.len());

    // change a list of SymbolInfo into a 2D list of WSTopic per session in max 100 index
    let subs = format_subscription_list(&symbol_infos);
    log::info!("Total orderbook WS sessions: {:?}", subs.len());

    // Create broadcast channels
    // for syncing
    let (tx_orderbook, rx_orderbook) = channel::<OrderbookEvent>(1024 * 2);
    // for getting notable orderbook after syncing
    let (tx_orderbook_best, rx_orderbook_best) = channel::<OrderbookEvent>(512);
    // for getting chance
    let (tx_chance, rx_chance) = channel::<ChanceEvent>(64);
    // for placing order
    let (tx_order, rx_order) = channel::<OrderEvent>(16);
    log::info!("Broadcast channels setup");

    let full_orderbook = Arc::new(Mutex::new(FullOrderbook::new()));
    log::info!("Local orderbook setup");

    // Infrastructure tasks
    tokio::spawn(task_sync_orderbook(
        rx_orderbook,
        tx_orderbook_best,
        full_orderbook.clone(),
    ));
    tokio::spawn(task_pub_chance_all_taker_btc_usd(
        rx_orderbook_best,
        tx_chance,
        full_orderbook.clone(),
        hash_symbols,
    ));
    tokio::spawn(task_gatekeep_chances(rx_chance, tx_order));
    tokio::spawn(task_place_order(rx_order, api.clone()));

    // extract the names only
    let symbols: Vec<String> = symbol_infos.into_iter().map(|info| info.symbol).collect();

    // TODO place below in broker
    // use REST to obtain the initial orderbook before subscribing to websocket
    let tasks: Vec<_> = symbols
        .iter()
        .map(|symbol| {
            // clone variables per task before spawn
            let api = api.clone();
            let full_orderbook_2 = full_orderbook.clone();
            let symbol = symbol.clone();

            tokio::spawn(async move {
                log::info!("Obtaining initial orderbook[{}] from REST", symbol);
                // OrderBookType::Full fails
                let res = api
                    .get_orderbook(&symbol, OrderBookType::L100)
                    .await
                    .expect("invalid data");

                if let Some(data) = res.data {
                    log::info!("Initial sequence {}:{}", &symbol, data.sequence);
                    let mut x = full_orderbook_2.lock().await;
                    x.insert(symbol.to_string(), data.to_internal());
                } else {
                    log::warn!("orderbook[{}] received none", &symbol);
                }
            })
        })
        .collect();
    futures::future::join_all(tasks).await;
    log::info!("collected all the symbols");

    // setup subscription and tasks per session, this is the source of data
    for (i, sub) in subs.iter().enumerate() {
        let mut ws = api.websocket();
        ws.subscribe(url.clone(), sub.clone()).await?;
        // TODO change to task_pub_orderbook_event
        tokio::spawn(task_pub_orderbook_event(ws, tx_orderbook.clone()));
        log::info!("{i:?}-th session of WS subscription setup");
    }

    log::info!("all application tasks setup");
    let _res = tokio::join!(kucoin_arbitrage::global::task::background_routine());
    panic!("Program should not arrive here")
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
