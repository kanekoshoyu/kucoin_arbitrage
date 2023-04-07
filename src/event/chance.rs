use crate::model::chance::TriangularArbitrageChance;

//TODO implement arbitrage chance events below
#[derive(Debug, Clone)]
pub enum ChanceEvent {
    AllTaker(TriangularArbitrageChance),
    MakerTakerTaker(TriangularArbitrageChance),
}
