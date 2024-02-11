use std::sync::Arc;

use eyre::Result;
/// Test latency between order and private channel order detection
/// Places extreme order in REST, receive extreme order in private channel
/// Please configure the buy price to either the current market price or lower for testing purpose
use kucoin_api::client::{Kucoin, KucoinEnv};
use kucoin_arbitrage::broker::gatekeeper::kucoin::task_gatekeep_chances;
use kucoin_arbitrage::broker::order::kucoin::task_place_order;
use kucoin_arbitrage::broker::symbol::kucoin::{format_subscription_list, get_symbols};
use kucoin_arbitrage::broker::trade::kucoin::task_pub_trade_event;
use kucoin_arbitrage::event::chance::ChanceEvent;
use kucoin_arbitrage::event::order::OrderEvent;
use kucoin_arbitrage::event::trade::TradeEvent;
use kucoin_arbitrage::model::chance::{ActionInfo, TriangularArbitrageChance};
use kucoin_arbitrage::monitor::counter::Counter;
use kucoin_arbitrage::monitor::task::{task_log_mps, task_monitor_channel_mps};
use kucoin_arbitrage::system_event::task_signal_handle;
use kucoin_arbitrage::{broker::symbol::filter::symbol_with_quotes, monitor};
use ordered_float::OrderedFloat;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<()> {
    // Provides logging format
    // kucoin_arbitrage::logger::log_init()?;
    tracing::info!("Log setup");

    // config
    let config = kucoin_arbitrage::config::from_file("config.toml")?;

    let api = Kucoin::new(KucoinEnv::Live, Some(config.kucoin_credentials()))
        .map_err(|e| eyre::eyre!(e))?;
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
    let cx_chance = Arc::new(Mutex::new(Counter::new("chance")));
    let tx_chance = broadcast::channel::<ChanceEvent>(32).0;
    let cx_order = Arc::new(Mutex::new(Counter::new("order")));
    let tx_order = broadcast::channel::<OrderEvent>(16).0;
    let cx_trade = Arc::new(Mutex::new(Counter::new("trade")));
    let tx_trade = broadcast::channel::<TradeEvent>(32).0;
    tracing::info!("Broadcast channels setup");

    // monitor tasks
    let mut taskpool_monitor = JoinSet::new();
    taskpool_monitor.spawn(task_monitor_channel_mps(
        tx_chance.subscribe(),
        cx_chance.clone(),
    ));
    taskpool_monitor.spawn(task_monitor_channel_mps(
        tx_order.subscribe(),
        cx_order.clone(),
    ));
    taskpool_monitor.spawn(task_monitor_channel_mps(
        tx_trade.subscribe(),
        cx_trade.clone(),
    ));
    taskpool_monitor.spawn(task_log_mps(
        vec![cx_chance.clone(), cx_order.clone(), cx_trade.clone()],
        10,
    ));

    let mut taskpool_infrastructure: JoinSet<Result<()>> = JoinSet::new();
    taskpool_infrastructure.spawn(task_place_order(tx_order.subscribe(), api.clone()));
    taskpool_infrastructure.spawn(task_gatekeep_chances(
        tx_chance.subscribe(),
        tx_trade.subscribe(),
        tx_order.clone(),
    ));
    taskpool_infrastructure.spawn(task_pub_trade_event(api.clone(), tx_trade.clone()));

    tracing::info!("All application tasks setup");
    monitor::timer::start("order_placement_network".to_string()).await;
    monitor::timer::start("order_placement_broadcast".to_string()).await;
    let err_msg = tokio::select! {
        _ = task_pub_chance_periodically(tx_chance.clone(), 15.0) => format!("failed publishing"),
        res = taskpool_infrastructure.join_next() => format!("taskpool_infrastructure stopped unexpectedly [{res:?}]"),
        _ = task_signal_handle() => format!("Received external signal, terminating program"),
    };
    tracing::error!("{err_msg}");
    tracing::info!("Exiting program, bye!");
    Ok(())
}

async fn task_pub_chance_periodically(
    tx_chance: broadcast::Sender<ChanceEvent>,
    interval_s: f64,
) -> Result<()> {
    // unique ID to be generated every time
    loop {
        let chance = TriangularArbitrageChance {
            profit: OrderedFloat::from(0.1),
            actions: [
                ActionInfo::buy("BTC-USDT".to_string(), OrderedFloat(0.1), OrderedFloat(0.1)),
                ActionInfo::buy("ETH-BTC".to_string(), OrderedFloat(0.1), OrderedFloat(0.1)),
                ActionInfo::sell("ETH-USDT".to_string(), OrderedFloat(0.1), OrderedFloat(0.1)),
            ],
        };
        let event = ChanceEvent::AllTaker(chance);
        tx_chance.send(event.clone())?;
        tokio::time::sleep(tokio::time::Duration::from_secs_f64(interval_s)).await;
    }
}
