#[derive(Debug, Clone, Copy)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl Default for OrderSide {
    fn default() -> Self {
        OrderSide::Sell
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

pub trait Order {
    fn id(&self) -> u128;
    fn side(&self) -> OrderSide;
    fn currency(&self) -> String;
    fn amount(&self) -> String;
    fn order_type(&self) -> OrderType;
}

#[derive(Debug, Clone)]
pub struct MarketOrder {
    id: u128,
    order_type: OrderType,
    side: OrderSide,
    currency: String,
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
    fn currency(&self) -> String {
        self.currency.clone()
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
    currency: String,
    amount: String,
    size: String,
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
    fn currency(&self) -> String {
        self.currency.clone()
    }
    fn amount(&self) -> String {
        self.amount.clone()
    }
}

impl LimitOrder {
    pub fn size(&self) -> String {
        self.size.clone()
    }
}
