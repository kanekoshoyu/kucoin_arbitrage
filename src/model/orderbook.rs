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

    pub fn merge(&mut self, to_merge: Orderbook) -> Result<(), ()> {
        if self.sequence > to_merge.sequence || self.time > to_merge.time {
            return Err(());
        }
        // make sure that to_merge's PVMaps are already filtered such that
        // it is all behind the starting sequence
        self.sequence = to_merge.sequence;
        self.time = to_merge.time;
        for (price, volume) in to_merge.ask.into_iter() {
            self.ask.insert(price, volume);
        }
        for (price, volume) in to_merge.bid.into_iter() {
            self.bid.insert(price, volume);
        }
        Ok(())
    }
}
