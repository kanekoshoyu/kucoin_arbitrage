use std::collections::HashMap;

pub type PVMap = HashMap<String, String>; //Prices to Volume
#[derive(Debug, Clone)]
pub struct Orderbook {
    pub ask: PVMap,
    pub bid: PVMap,
    pub time: i64,
    pub sequence: u64,
}

pub type FullOrderbook = HashMap<String, Orderbook>; //Symbols to Orderbook

impl Orderbook {
    pub fn new() -> Orderbook {
        Orderbook {
            ask: PVMap::new(),
            bid: PVMap::new(),
            time: chrono::offset::Utc::now().timestamp(),
            sequence: 0,
        }
    }
}
