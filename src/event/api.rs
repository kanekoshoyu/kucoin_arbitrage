use crate::model::orderbook::Orderbook;

#[derive(Debug, Clone)]
pub enum ApiEvent {
    OrderbookReceived(Orderbook),
    OrderbookChangeReceived(Orderbook),
}
