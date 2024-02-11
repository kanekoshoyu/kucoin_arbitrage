use std::sync::Arc;

use eyre::Result;
/// Test latency between order and private channel order detection
/// Places extreme order in REST, receive extreme order in private channel
/// Please configure the buy price to either the current market price or lower for testing purpose
use kucoin_api::client::{Kucoin, KucoinEnv};
use kucoin_arbitrage::broker::order::kucoin::task_place_order;
use kucoin_arbitrage::broker::symbol::kucoin::{format_subscription_list, get_symbols};
use kucoin_arbitrage::broker::trade::kucoin::task_pub_trade_event;
use kucoin_arbitrage::event::order::OrderEvent;
use kucoin_arbitrage::event::trade::TradeEvent;
use kucoin_arbitrage::model::order::{LimitOrder, OrderSide, OrderType};
use kucoin_arbitrage::monitor::counter::Counter;
use kucoin_arbitrage::monitor::task::{task_log_mps, task_monitor_channel_mps};
use kucoin_arbitrage::{broker::symbol::filter::symbol_with_quotes, monitor};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinSet;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Provides logging format
    kucoin_arbitrage::logger::log_init()?;
    tracing::info!("Log setup");

    // config
    let config = kucoin_arbitrage::config::from_file("config.toml")?;

    let api = Kucoin::new(KucoinEnv::Live, Some(config.kucoin_credentials()))?;
    tracing::info!("Credentials setup");

    // Gets all symbols concurrently
    let symbol_list = get_symbols(api.clone()).await;
    tracing::info!("Total exchange symbols: {:?}", symbol_list.len());

    // Filters with either btc or usdt as quote
    let symbol_infos = symbol_with_quotes(&symbol_list, "BTC", "USDT");
    tracing::info!("Total symbols in scope: {:?}", symbol_infos.len());

    // Changes a list of SymbolInfo into a 2D list of WSTopic per session in max 100 index
    let subs = format_subscription_list(&symbol_infos);
    tracing::info!("Total orderbook WS sessions: {:?}", subs.len());

    // Creates broadcast channels
    let cx_order = Arc::new(Mutex::new(Counter::new("order")));
    let tx_order = broadcast::channel::<OrderEvent>(16).0;
    let cx_trade = Arc::new(Mutex::new(Counter::new("trade")));
    let tx_trade = broadcast::channel::<TradeEvent>(128).0;
    tracing::info!("Broadcast channels setup");

    // monitor tasks
    let mut taskpool_monitor = JoinSet::new();
    taskpool_monitor.spawn(task_monitor_channel_mps(
        tx_order.subscribe(),
        cx_order.clone(),
    ));
    taskpool_monitor.spawn(task_monitor_channel_mps(
        tx_trade.subscribe(),
        cx_trade.clone(),
    ));
    taskpool_monitor.spawn(task_log_mps(vec![cx_order.clone(), cx_trade.clone()], 10));

    let mut taskpool_infrastructure: JoinSet<Result<()>> = JoinSet::new();
    taskpool_infrastructure.spawn(task_place_order(tx_order.subscribe(), api.clone()));
    taskpool_infrastructure.spawn(task_pub_trade_event(api.clone(), tx_trade.clone()));

    tracing::info!("All application tasks setup");
    monitor::timer::start("order_placement_network".to_string()).await;
    monitor::timer::start("order_placement_broadcast".to_string()).await;
    let err_msg = tokio::select! {
        res = task_place_order_periodically(tx_order.clone(), 15.0) => format!("failed placing order [{res:?}]"),
        res = taskpool_infrastructure.join_next() => {
            let res = res.unwrap();
            format!("taskpool_infrastructure stopped unexpectedly [{res:?}]")
        },
        _ = task_signal_handle() => format!("Received external signal, terminating program"),
    };
    tracing::error!("{err_msg}");
    tracing::info!("Exiting program, bye!");
    Ok(())
}

/// wait for any external terminating signal
async fn task_signal_handle() -> Result<()> {
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    tokio::select! {
        _ = sigterm.recv() => exit_program("SIGTERM").await?,
        _ = sigint.recv() => exit_program("SIGINT").await?,
    };
    Ok(())
}

/// handle external signal
async fn exit_program(signal_alias: &str) -> Result<()> {
    tracing::info!("Received [{signal_alias}] signal");
    Ok(())
}

async fn task_place_order_periodically(
    tx_order: broadcast::Sender<OrderEvent>,
    interval_s: f64,
) -> Result<()> {
    // unique ID to be generated every time
    loop {
        let event = OrderEvent::PlaceLimitOrder(LimitOrder {
            id: Uuid::new_v4().to_string(),
            order_type: OrderType::Limit,
            side: OrderSide::Buy,
            symbol: "BTC-USDT".to_string(),
            amount: 0.0001.to_string(),
            price: 35000.0.to_string(),
        });
        tx_order.send(event.clone())?;
        tokio::time::sleep(tokio::time::Duration::from_secs_f64(interval_s)).await;
    }
}
