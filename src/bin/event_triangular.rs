/// Executes triangular arbitrage
use kucoin_api::client::{Kucoin, KucoinEnv};
use kucoin_api::model::market::{OrderBook, OrderBookType};
use kucoin_arbitrage::broker::gatekeeper::kucoin::task_gatekeep_chances;
use kucoin_arbitrage::broker::order::kucoin::task_place_order;
use kucoin_arbitrage::broker::orderbook::kucoin::{task_pub_orderbook_event, task_sync_orderbook};
use kucoin_arbitrage::broker::orderchange::kucoin::task_pub_orderchange_event;
use kucoin_arbitrage::broker::symbol::filter::{symbol_with_quotes, vector_to_hash};
use kucoin_arbitrage::broker::symbol::kucoin::{format_subscription_list, get_symbols};
use kucoin_arbitrage::event::{
    chance::ChanceEvent, order::OrderEvent, orderbook::OrderbookEvent,
    orderchange::OrderChangeEvent,
};
use kucoin_arbitrage::global::task::task_log_mps;
use kucoin_arbitrage::model::{counter::Counter, orderbook::FullOrderbook};
use kucoin_arbitrage::strategy::all_taker_btc_usd::task_pub_chance_all_taker_btc_usd;
use kucoin_arbitrage::translator::traits::OrderBookTranslator;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::broadcast::channel;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Log setup");

    // credentials
    let config = kucoin_arbitrage::config::from_file("config.toml")?;

    tokio::select! {
        _ = task_signal_handle() => println!("received external signal, terminating program"),
        res = core(config) => println!("core ended first {res:?}"),
    };

    println!("Good bye!");
    Ok(())
}

async fn core(config: kucoin_arbitrage::config::Config) -> Result<(), failure::Error> {
    // config parameters
    let budget = config.behaviour.usd_cyclic_arbitrage;
    let monitor_interval = config.behaviour.monitor_interval_sec;

    // system mps counters
    let api_input_counter = Arc::new(Mutex::new(Counter::new("api_input")));
    let best_price_counter = Arc::new(Mutex::new(Counter::new("best_price")));
    let chance_counter = Arc::new(Mutex::new(Counter::new("chance")));
    let order_counter = Arc::new(Mutex::new(Counter::new("order")));

    // api endpoints
    let api = Kucoin::new(KucoinEnv::Live, Some(config.kucoin_credentials()))?;
    log::info!("Credentials setup");

    // gets all symbols concurrently
    let symbol_list = get_symbols(api.clone()).await;
    log::info!("Total exchange symbols: {:?}", symbol_list.len());

    // filters with either btc or usdt as quote
    let symbol_infos = symbol_with_quotes(&symbol_list, "BTC", "USDT");
    let hash_symbols = Arc::new(Mutex::new(vector_to_hash(&symbol_infos)));
    log::info!("Total symbols in scope: {:?}", symbol_infos.len());

    // list subscription using the filtered symbols
    let subs = format_subscription_list(&symbol_infos);
    log::info!("Total orderbook WS sessions: {:?}", subs.len());

    // creates broadcast channels
    // for syncing public orderbook
    let (tx_orderbook, rx_orderbook) = channel::<OrderbookEvent>(1024 * 2);
    // for getting notable orderbook after syncing
    let (tx_orderbook_best, rx_orderbook_best) = channel::<OrderbookEvent>(512);
    // for getting chance
    let (tx_chance, rx_chance) = channel::<ChanceEvent>(64);
    // for placing order
    let (tx_order, rx_order) = channel::<OrderEvent>(16);
    // for getting private order changes
    let (tx_orderchange, rx_orderchange) = channel::<OrderChangeEvent>(128);
    log::info!("Broadcast channels setup");

    // creates local orderbook
    let full_orderbook = Arc::new(Mutex::new(FullOrderbook::new()));
    log::info!("Local empty full orderbook setup");

    // infrastructure tasks
    let mut taskpool_infrastructure = JoinSet::new();

    // USD cyclic arbitrage budget obtained from CONFIG
    taskpool_infrastructure.spawn(task_sync_orderbook(
        rx_orderbook,
        tx_orderbook_best,
        full_orderbook.clone(),
        api_input_counter.clone(),
    ));
    taskpool_infrastructure.spawn(task_pub_chance_all_taker_btc_usd(
        rx_orderbook_best,
        tx_chance,
        full_orderbook.clone(),
        hash_symbols,
        budget as f64,
        best_price_counter.clone(),
    ));
    taskpool_infrastructure.spawn(task_gatekeep_chances(
        rx_chance,
        rx_orderchange,
        tx_order,
        chance_counter.clone(),
    ));
    taskpool_infrastructure.spawn(task_place_order(
        rx_order,
        api.clone(),
        order_counter.clone(),
    ));
    taskpool_infrastructure.spawn(task_log_mps(
        vec![
            api_input_counter.clone(),
            best_price_counter.clone(),
            chance_counter.clone(),
            order_counter.clone(),
        ],
        monitor_interval as u64,
    ));

    // TODO wrap below into a task and join them all using taskpool
    // collect all initial orderbook states with REST
    aggregate_orderbook_state(api.clone(), symbol_infos, full_orderbook).await?;
    log::info!("Aggregated all the symbols");

    let mut taskpool_subscription = JoinSet::new();
    taskpool_subscription.spawn(task_pub_orderchange_event(api.clone(), tx_orderchange));

    for (i, sub) in subs.iter().enumerate() {
        taskpool_subscription.spawn(task_pub_orderbook_event(
            api.clone(),
            sub.to_vec(),
            tx_orderbook.clone(),
        ));
        log::info!("{i:?}-th session of WS subscription setup");
    }
    let message = tokio::select! {
        res = taskpool_infrastructure.join_next() =>
            format!("Infrastructure task pool error [{res:?}]"),
        res = taskpool_subscription.join_next() => format!("Subscription task pool error [{res:?}]"),
    };
    Err(failure::err_msg(format!("unexpected error [{message}]")))
}

/// task to wait for any external terminating signal
async fn task_signal_handle() -> Result<(), failure::Error> {
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    tokio::select! {
        _ = sigterm.recv() => exit_program("SIGTERM").await?,
        _ = sigint.recv() => exit_program("SIGINT").await?,
    };
    Ok(())
}

// Define a handler function for the SIGTERM signal
async fn exit_program(signal_alias: &str) -> Result<(), failure::Error> {
    log::info!("Received [{signal_alias}] signal");
    Ok(())
}

async fn aggregate_orderbook_state(
    api: Kucoin,
    symbol_infos: Vec<kucoin_arbitrage::model::symbol::SymbolInfo>,
    full_orderbook: Arc<Mutex<FullOrderbook>>,
) -> Result<(), failure::Error> {
    // replace spawn with or a taskpool
    let mut taskpool_aggregate = JoinSet::new();
    // collect all initial orderbook states with REST
    let symbols: Vec<String> = symbol_infos.into_iter().map(|info| info.symbol).collect();
    log::info!("Total symbols: {:?}", symbols.len());
    for symbol in symbols {
        let api = api.clone();
        let full_orderbook_arc = full_orderbook.clone();
        taskpool_aggregate.spawn(async move {
            let data = task_get_orderbook(api, &symbol).await.unwrap();
            let mut x = full_orderbook_arc.lock().await;
            x.insert(symbol.to_string(), data.to_internal());
            symbol
        });
        // prevent server overloading
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    while let Some(res) = taskpool_aggregate.join_next().await {
        if let Err(e) = res {
            return Err(failure::Error::from(e));
        }
        log::info!("Initialized orderbook for [{:?}]", res.unwrap());
    }
    Ok(())
}

async fn task_get_orderbook(api: Kucoin, symbol: &str) -> Result<OrderBook, failure::Error> {
    let mut try_counter = 0;
    loop {
        try_counter += 1;
        // OrderBookType::Full requires valid API Key
        let res = api.get_orderbook(symbol, OrderBookType::L20).await;
        if res.is_err() {
            log::warn!("orderbook[{symbol}] did not respond ({try_counter:?} tries)");
            continue;
        }
        let res = res.unwrap();
        let code = res.code;
        match code.as_str() {
            "429000" => {
                log::warn!("[{symbol:?}] request overloaded ({try_counter:?} tries)")
            }
            "200000" => {
                if res.data.is_none() {
                    log::warn!("orderbook[{symbol}] received none ({try_counter:?} tries)");
                    continue;
                }
                log::info!("obtained [{symbol}]");
                return Ok(res.data.unwrap());
            }
            "400003" => return Err(failure::err_msg(format!("API key needed not but provided"))),
            _ => return Err(failure::err_msg(format!("unrecognised code [{:?}]", code))),
        }
    }
}
