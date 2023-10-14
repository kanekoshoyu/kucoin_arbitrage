/// Test latency between order and private channel order detection
/// Places extreme order in REST, receive extreme order in private channel
/// Please configure the buy price to either the current market price or lower for testing purpose
use kucoin_api::client::{Kucoin, KucoinEnv};
use kucoin_arbitrage::broker::order::kucoin::task_place_order;
use kucoin_arbitrage::broker::orderchange::kucoin::task_pub_orderchange_event;
use kucoin_arbitrage::broker::symbol::kucoin::{format_subscription_list, get_symbols};
use kucoin_arbitrage::event::order::OrderEvent;
use kucoin_arbitrage::event::orderchange::OrderChangeEvent;
use kucoin_arbitrage::model::counter::Counter;
use kucoin_arbitrage::model::order::{LimitOrder, OrderSide, OrderType};
use kucoin_arbitrage::strings::generate_uid;
use kucoin_arbitrage::{broker::symbol::filter::symbol_with_quotes, monitor};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::channel;
use tokio::sync::Mutex;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // Provides logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Log setup");

    // Declares all the system counters
    let order_counter = Arc::new(Mutex::new(Counter::new("order")));

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
    // for placing order
    let (tx_order, rx_order) = channel::<OrderEvent>(16);
    // for getting private order changes
    let (tx_orderchange, _rx_orderchange) = channel::<OrderChangeEvent>(128);
    log::info!("Broadcast channels setup");

    // TODO use the tx_order to send orders
    tokio::spawn(task_place_order(
        rx_order,
        api.clone(),
        order_counter.clone(),
    ));

    tokio::spawn(task_pub_orderchange_event(api.clone(), tx_orderchange));

    log::info!("All application tasks setup");

    monitor::timer::start("order_placement_network".to_string()).await;
    monitor::timer::start("order_placement_broadcast".to_string()).await;
    // Sends a post order
    let event = OrderEvent::PostOrder(LimitOrder {
        id: generate_uid(40),
        order_type: OrderType::Limit,
        side: OrderSide::Buy,
        symbol: "BTC-USDT".to_string(),
        amount: 0.001.to_string(),
        price: 29947.0.to_string(),
    });
    if let Err(e) = tx_order.send(event) {
        log::error!("{e}");
    }

    loop {
        // Waits 60 seconds
        sleep(Duration::from_secs(60)).await;
    }
}
