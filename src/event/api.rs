use crate::model::orderbook::Orderbook;

#[derive(Debug, Clone)]
pub enum ApiEvent {
    OrderbookReceived((String, Orderbook)),
    OrderbookChangeReceived((String, Orderbook)),
}
