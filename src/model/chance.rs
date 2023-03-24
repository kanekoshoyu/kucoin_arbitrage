use crate::model::order::OrderSide;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActionInfo {
    pub action: OrderSide,
    pub ticker: String,
    pub volume: String,
}

// sequence in ascending order
pub type ThreeActions = [ActionInfo; 3];
