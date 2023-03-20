use crate::model::order::LimitOrder;

#[derive(Debug, Clone)]
pub enum OrderEvent {
    GetAllOrders,
    CancelOrder(LimitOrder),
    CancelAllOrders,
    PostOrder(LimitOrder),
}
