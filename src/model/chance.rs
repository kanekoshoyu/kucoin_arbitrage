use crate::model::order::OrderSide;

/// structure of of arbitrage chances
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActionInfo {
    pub action: OrderSide,
    pub ticker: String,
    pub volume: String,
}

// sequence in ascending order
pub type ThreeActions = [ActionInfo; 3];

/// structure of of arbitrage chances
#[derive(Debug, Clone, Default)]
pub struct TriangularArbitrageChance {
    pub profit: f32,
    pub actions: ThreeActions,
}
