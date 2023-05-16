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
use kucoin_arbitrage::strategy::all_taker_btc_usd::task_pub_chance_all_taker_btc_usd;
use kucoin_arbitrage::translator::traits::OrderBookTranslator;
use std::sync::Arc;
use tokio::sync::broadcast::channel;
use tokio::sync::Mutex;

fn prune_vector<T>(input_vec: Vec<T>, n: usize) -> Vec<T> {
    let mut output_vec = Vec::new();
    for (index, value) in input_vec.into_iter().enumerate() {
        if index >= n {
            break;
        }
        output_vec.push(value);
    }
    output_vec
}

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

    // TODO use tokio_spawn to get the below data concurrently
    // get symbol lists
    let symbol_list = get_symbols(api.clone()).await;
    let symbol_infos = symbol_with_quotes(&symbol_list, "BTC", "USDT");
    let hash_symbols = Arc::new(Mutex::new(vector_to_hash(&symbol_infos)));

    log::info!("Total symbols: {:?}", symbol_infos.len());

    // prune to smaller dataset for testing. size is 1+2N
    let symbol_infos = prune_vector(symbol_infos, 99);
    let mut symbols = Vec::new();
    for symbol_info in symbol_infos {
        symbols.push(symbol_info.symbol);
    }

    // Initialize the websocket
    let mut ws = api.websocket();
    let subs = vec![WSTopic::OrderBook(symbols.to_vec())];
    ws.subscribe(url, subs).await?;
    log::info!("Websocket subscription setup");

    // Create broadcast channels
    // for syncing
    let (tx_orderbook, rx_orderbook) = channel::<OrderbookEvent>(1024 * 2);
    // for getting notable orderbook after syncing
    let (tx_orderbook_best, rx_orderbook_best) = channel::<OrderbookEvent>(1024);
    // for getting chance
    let (tx_chance, rx_chance) = channel::<ChanceEvent>(64);
    // for placing order
    let (tx_order, rx_order) = channel::<OrderEvent>(16);
    log::info!("broadcast channels setup");

    let full_orderbook = Arc::new(Mutex::new(FullOrderbook::new()));
    log::info!("Local Orderbook setup");

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

    // use REST to obtain the initial orderbook before subscribing to websocket
    let full_orderbook_2 = full_orderbook.clone();

    for symbol in symbols.iter() {
        log::info!("btaining initial orderbook[{symbol}] from REST");
        // OrderBookType::Full fails
        let res = api
            .clone()
            .get_orderbook(symbol.as_str(), OrderBookType::L100)
            .await
            .expect("invalid data");
        if let Some(data) = res.data {
            // log::info!("orderbook[{symbol}] {:#?}", data);
            log::info!("Initial sequence {}:{}", symbol, data.sequence);
            let mut x = full_orderbook_2.lock().await;
            x.insert(symbol.to_string(), data.to_internal());
        } else {
            log::warn!("orderbook[{symbol}] received none")
        }
    }

    // task_pub_orderevent is the source of data (websocket)
    tokio::spawn(task_pub_orderbook_event(ws, tx_orderbook));
    log::info!("task_pub_orderevent setup");

    log::info!("all application tasks setup");
    let _res = tokio::join!(kucoin_arbitrage::global::task::background_routine());
    panic!("Program should not arrive here")
}
