use ordered_float::OrderedFloat;
use std::collections::BTreeMap;
use std::collections::HashMap;

/// price as key, volume as value
pub type PVMap = BTreeMap<OrderedFloat<f32>, OrderedFloat<f32>>; //Prices to Volume

/// orderbook for each symbol, contains ask, bid, time and sequence
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Orderbook {
    pub ask: PVMap,
    pub bid: PVMap,
    pub sequence: u64,
}

/// symbol as key, orderbook as value
pub type FullOrderbook = HashMap<String, Orderbook>; //Symbols to Orderbook

impl Orderbook {
    pub fn new() -> Orderbook {
        Orderbook {
            ask: PVMap::new(),
            bid: PVMap::new(),
            sequence: 0,
        }
    }

    pub fn merge(&mut self, to_merge: Orderbook) -> Result<(), ()> {
        if self.sequence > to_merge.sequence {
            log::warn!("merge skipped: orderbook.sequence:{:?} is merging into orderbook.sequence:{:?}\n", to_merge.sequence, self.sequence);
            return Err(());
        }
        // make sure that to_merge's PVMaps are already filtered such that
        // it is all behind the starting sequence
        self.sequence = to_merge.sequence;
        for (price, volume) in to_merge.ask.into_iter() {
            self.ask.insert(price, volume);
        }
        for (price, volume) in to_merge.bid.into_iter() {
            self.bid.insert(price, volume);
        }
        Ok(())
    }
}
