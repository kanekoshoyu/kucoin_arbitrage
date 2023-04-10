use crate::model::order::Order;

/// trait used by market gatekeeper to determine of the order is allowed
pub trait ExchangeLimit {
    fn is_allowed(&self, order: dyn Order) -> bool;
}
