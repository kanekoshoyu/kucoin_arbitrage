#[derive(Debug, Clone)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone)]
pub struct LimitOrder {
    pub id: String,
    pub side: Side,
    pub symbol: String,
    pub size: String,
    pub order_type: OrderType,
}
