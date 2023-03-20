use crate::model::orderbook::Orderbook;

#[derive(Debug, Clone)]
pub enum OrderbookEvent {
    OrderbookReceived((String, Orderbook)),
    OrderbookChangeReceived((String, Orderbook)),
}
