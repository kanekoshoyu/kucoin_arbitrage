/// Arbitrage chance enum for internal
pub mod chance;
/// Order placement enum for REST
pub mod order;
/// Orderbook change enum for subscription
pub mod orderbook;
/// Order change enum for subscription
pub mod orderchange;

/// Casts all events into generic event type to be used by the broadcast, more efficient than using dyn traits
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    ChanceEvent(chance::ChanceEvent),
    OrderEvent(order::OrderEvent),
    OrderbookEvent(orderbook::OrderbookEvent),
    OrderChangeEvent(orderchange::OrderChangeEvent),
}
