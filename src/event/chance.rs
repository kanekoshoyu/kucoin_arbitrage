use crate::model::chance::ThreeActions;

//TODO implement arbitrage chance events below
#[derive(Debug, Clone)]
pub enum ChanceEvent {
    AllTaker(ThreeActions),
    MakerTakerTaker(ThreeActions),
}
