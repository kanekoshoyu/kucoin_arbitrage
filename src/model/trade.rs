use crate::model::order;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TradeInfo {
    pub order_id: u128,
    pub symbol: String,
    pub side: order::OrderSide,
    pub order_type: order::OrderType,
    pub size: String,
}
