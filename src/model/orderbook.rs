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

    pub fn merge(&mut self, to_merge: Orderbook) -> Result<Option<Orderbook>, String> {
        let to_merge_clone = to_merge.clone();
        let min_ask = self.ask.first_key_value().unwrap().0.to_owned();
        let max_bid = self.bid.last_key_value().unwrap().0.to_owned();

        if self.sequence > to_merge.sequence {
            return Err(String::from(
                "older orderbook trying to get merged into newer orderbook",
            ));
        }
        // make sure that to_merge's PVMaps are already filtered such that
        // it is all behind the starting sequence
        self.sequence = to_merge.sequence;
        // return the value when to_merge is the best (i.e. lowest ask or highest bid)
        // TODO find breaking record here

        for (price, volume) in to_merge.ask.into_iter() {
            self.ask.insert(price, volume);
        }
        for (price, volume) in to_merge.bid.into_iter() {
            self.bid.insert(price, volume);
        }

        if let Some((merge_min_ask, _)) = to_merge_clone.ask.first_key_value() {
            if merge_min_ask.to_owned() < min_ask {
                return Ok(Some(to_merge_clone));
            }
        }
        if let Some((merge_max_bid, _)) = to_merge_clone.bid.first_key_value() {
            if merge_max_bid.to_owned() > max_bid {
                return Ok(Some(to_merge_clone));
            }
        }
        // let (merge_min_ask, _) = to_merge_clone.ask.first_key_value().unwrap();
        // let (merge_max_bid, _) = to_merge_clone.bid.last_key_value().unwrap();
        // if merge_min_ask < min_ask || merge_max_bid > max_bid {
        //     Some(to_merge_clone);
        // }
        Ok(None)
    }
}
