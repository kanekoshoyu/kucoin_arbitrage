use crate::model::orderbook::Orderbook;

/// public orderbook change received from exchange
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderbookEvent {
    OrderbookReceived((String, Orderbook)),
    OrderbookChangeReceived((String, Orderbook)),
}
