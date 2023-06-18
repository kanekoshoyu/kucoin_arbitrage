use crate::model::chance::TriangularArbitrageChance;

/// Arbitrage chance, wraps a chance model
#[derive(Debug, Clone)]
pub enum ChanceEvent {
    AllTaker(TriangularArbitrageChance),
    MakerTakerTaker(TriangularArbitrageChance),
}
