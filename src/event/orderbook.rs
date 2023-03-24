use crate::model::orderbook::Orderbook;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderbookEvent {
    OrderbookReceived((String, Orderbook)),
    OrderbookChangeReceived((String, Orderbook)),
}
