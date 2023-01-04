#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct ActionInfo {
    pub action: Action,
    pub ticker: String,
    pub volume: String,
}

// sequence in ascending order
pub type ActionSequence = [ActionInfo; 3];