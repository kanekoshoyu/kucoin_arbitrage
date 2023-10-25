use std::sync::Arc;

/// Test latency between order and private channel order detection
/// Places extreme order in REST, receive extreme order in private channel
/// Please configure the buy price to either the current market price or lower for testing purpose
use kucoin_api::client::{Kucoin, KucoinEnv};
use kucoin_arbitrage::broker::order::kucoin::task_place_order;
use kucoin_arbitrage::broker::orderchange::kucoin::task_pub_orderchange_event;
use kucoin_arbitrage::broker::symbol::kucoin::{format_subscription_list, get_symbols};
use kucoin_arbitrage::event::order::OrderEvent;
use kucoin_arbitrage::event::orderchange::OrderChangeEvent;
use kucoin_arbitrage::model::order::{LimitOrder, OrderSide, OrderType};
use kucoin_arbitrage::monitor::counter::Counter;
use kucoin_arbitrage::monitor::task::{task_log_mps, task_monitor_channel_mps};
use kucoin_arbitrage::strings::generate_uid;
use kucoin_arbitrage::{broker::symbol::filter::symbol_with_quotes, monitor};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // Provides logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Log setup");

    // config
    let config = kucoin_arbitrage::config::from_file("config.toml")?;

    let api = Kucoin::new(KucoinEnv::Live, Some(config.kucoin_credentials()))?;
    log::info!("Credentials setup");

    // Gets all symbols concurrently
    let symbol_list = get_symbols(api.clone()).await;
    log::info!("Total exchange symbols: {:?}", symbol_list.len());

    // Filters with either btc or usdt as quote
    let symbol_infos = symbol_with_quotes(&symbol_list, "BTC", "USDT");
    log::info!("Total symbols in scope: {:?}", symbol_infos.len());

    // Changes a list of SymbolInfo into a 2D list of WSTopic per session in max 100 index
    let subs = format_subscription_list(&symbol_infos);
    log::info!("Total orderbook WS sessions: {:?}", subs.len());

    // Creates broadcast channels
    let cx_order = Arc::new(Mutex::new(Counter::new("order")));
    let tx_order = broadcast::channel::<OrderEvent>(16).0;
    let cx_orderchange = Arc::new(Mutex::new(Counter::new("orderchange")));
    let tx_orderchange = broadcast::channel::<OrderChangeEvent>(128).0;
    log::info!("Broadcast channels setup");

    // monitor tasks
    let mut taskpool_monitor = JoinSet::new();
    taskpool_monitor.spawn(task_monitor_channel_mps(
        tx_order.subscribe(),
        cx_order.clone(),
    ));
    taskpool_monitor.spawn(task_monitor_channel_mps(
        tx_orderchange.subscribe(),
        cx_orderchange.clone(),
    ));
    taskpool_monitor.spawn(task_log_mps(
        vec![cx_order.clone(), cx_orderchange.clone()],
        10,
    ));

    let mut taskpool_infrastructure: JoinSet<Result<(), failure::Error>> = JoinSet::new();
    taskpool_infrastructure.spawn(task_place_order(tx_order.subscribe(), api.clone()));
    taskpool_infrastructure.spawn(task_place_order_periodically(tx_order.clone(), 10.0));
    taskpool_infrastructure.spawn(task_pub_orderchange_event(
        api.clone(),
        tx_orderchange.clone(),
    ));

    log::info!("All application tasks setup");
    monitor::timer::start("order_placement_network".to_string()).await;
    monitor::timer::start("order_placement_broadcast".to_string()).await;
    let err_msg = tokio::select! {
        res = taskpool_infrastructure.join_next() => format!("taskpool_infrastructure stopped unexpectedly [{res:?}]"),
        res = task_signal_handle() => format!("received external signal, terminating program [{res:?}]"),
    };
    log::warn!("{err_msg:?}");
    Ok(())
}

/// wait for any external terminating signal
async fn task_signal_handle() -> Result<(), failure::Error> {
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    tokio::select! {
        _ = sigterm.recv() => exit_program("SIGTERM").await?,
        _ = sigint.recv() => exit_program("SIGINT").await?,
    };
    Ok(())
}

/// handle external signal
async fn exit_program(signal_alias: &str) -> Result<(), failure::Error> {
    log::info!("Received [{signal_alias}] signal");
    Ok(())
}

async fn task_place_order_periodically(
    tx_order: broadcast::Sender<OrderEvent>,
    interval_s: f64,
) -> Result<(), failure::Error> {
    let event = OrderEvent::PlaceOrder(LimitOrder {
        id: generate_uid(40),
        order_type: OrderType::Limit,
        side: OrderSide::Buy,
        symbol: "BTC-USDT".to_string(),
        amount: 0.001.to_string(),
        price: 35000.0.to_string(),
    });
    loop {
        tx_order.send(event.clone())?;
        tokio::time::sleep(tokio::time::Duration::from_secs_f64(interval_s)).await;
    }
}
