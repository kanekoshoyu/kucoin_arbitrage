#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl Default for OrderSide {
    fn default() -> Self {
        OrderSide::Sell
    }
}

impl AsRef<str> for OrderSide {
    fn as_ref(&self) -> &str {
        match self {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        }
    }
}

/// ```
/// use kucoin_arbitrage::model::order::OrderSide;
/// let buy = OrderSide::Buy;
/// assert_eq!(buy.to_string(), "buy");
/// ```
impl ToString for OrderSide {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OrderType {
    Limit,
    Market,
}

impl Default for OrderType {
    fn default() -> Self {
        OrderType::Limit
    }
}

impl ToString for OrderType {
    fn to_string(&self) -> String {
        match self {
            OrderType::Limit => "limit",
            OrderType::Market => "market",
        }
        .to_string()
    }
}

pub trait Order {
    fn id(&self) -> u128;
    fn side(&self) -> OrderSide;
    fn symbol(&self) -> String;
    fn amount(&self) -> String;
    fn order_type(&self) -> OrderType;
}

#[derive(Debug, Clone)]
pub struct MarketOrder {
    id: u128,
    order_type: OrderType,
    side: OrderSide,
    symbol: String,
    amount: String,
}

impl Order for MarketOrder {
    fn id(&self) -> u128 {
        self.id.clone()
    }
    fn order_type(&self) -> OrderType {
        self.order_type.clone()
    }
    fn side(&self) -> OrderSide {
        self.side.clone()
    }
    fn symbol(&self) -> String {
        self.symbol.clone()
    }
    fn amount(&self) -> String {
        self.amount.clone()
    }
}

#[derive(Debug, Clone)]
pub struct LimitOrder {
    id: u128,
    order_type: OrderType,
    side: OrderSide,
    symbol: String,
    amount: String,
    price: String,
}

impl Order for LimitOrder {
    fn id(&self) -> u128 {
        self.id.clone()
    }
    fn order_type(&self) -> OrderType {
        self.order_type.clone()
    }
    fn side(&self) -> OrderSide {
        self.side.clone()
    }
    fn symbol(&self) -> String {
        self.symbol.clone()
    }
    fn amount(&self) -> String {
        self.amount.clone()
    }
}

impl LimitOrder {
    pub fn price(&self) -> String {
        self.price.clone()
    }
}
