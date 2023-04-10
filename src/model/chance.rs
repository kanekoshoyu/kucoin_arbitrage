use crate::model::order::OrderSide;
use ordered_float::OrderedFloat;

/// structure of of arbitrage chances
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActionInfo {
    pub action: OrderSide,
    pub ticker: String,
    pub volume: OrderedFloat<f32>,
}

impl ActionInfo {
    pub fn buy(ticker: String, volume: OrderedFloat<f32>) -> ActionInfo {
        ActionInfo {
            action: OrderSide::Buy,
            ticker,
            volume,
        }
    }
    pub fn sell(ticker: String, volume: OrderedFloat<f32>) -> ActionInfo {
        ActionInfo {
            action: OrderSide::Sell,
            ticker,
            volume,
        }
    }
}

// sequence in ascending order
pub type ThreeActions = [ActionInfo; 3];

/// structure of of arbitrage chances
#[derive(Debug, Clone, Default)]
pub struct TriangularArbitrageChance {
    pub profit: OrderedFloat<f32>,
    pub actions: ThreeActions,
}

// TODO implement order for TriangularArbitrageChance, using profit