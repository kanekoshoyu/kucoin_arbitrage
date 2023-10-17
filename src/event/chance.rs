use crate::model::chance::TriangularArbitrageChance;

/// Arbitrage chance, wraps a chance model
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChanceEvent {
    AllTaker(TriangularArbitrageChance),
    MakerTakerTaker(TriangularArbitrageChance),
}
