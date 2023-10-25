use crate::model::order::LimitOrder;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderEvent {
    GetAllOrders,
    CancelOrder(LimitOrder),
    CancelAllOrders,
    PlaceOrder(LimitOrder),
    PlaceBorrowOrder(LimitOrder),
}
