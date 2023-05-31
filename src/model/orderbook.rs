use ordered_float::OrderedFloat;
use std::collections::BTreeMap;
use std::collections::HashMap;

/// price as key, volume as value
pub type PVMap = BTreeMap<OrderedFloat<f64>, OrderedFloat<f64>>; //Prices to Volume

/// Internal printer struct
struct PVMapDebug<'a>(&'a PVMap);

impl<'a> std::fmt::Debug for PVMapDebug<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        for (price, volume) in self.0 {
            list.entry(&format_args!("{price}: {volume}"));
        }
        list.finish()
    }
}

/// orderbook for each symbol, contains ask, bid, time and sequence
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Orderbook {
    pub ask: PVMap,
    pub bid: PVMap,
    pub sequence: u64,
}

impl std::fmt::Debug for Orderbook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Orderbook")
            .field("sequence", &self.sequence)
            .field("ask", &PVMapDebug(&self.ask))
            .field("bid", &PVMapDebug(&self.bid))
            .finish()
    }
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

    /// Merges another Orderbook, returns Orderbook when there is increase in the best price (i.e. lowest ask or highest bid)
    pub fn merge(&mut self, to_merge: Orderbook) -> Result<Option<Orderbook>, String> {
        let to_merge_clone = to_merge.to_owned();

        // log::info!("to_merge: {to_merge_clone:?}");
        let zero = 0.0;

        // clone the best ask/bid
        let best_ask = self.ask.first_key_value().unwrap();
        let (min_ask, min_ask_volume) = (best_ask.0.to_owned(), best_ask.1.to_owned());
        let best_bid = self.bid.last_key_value().unwrap();
        let (max_bid, max_bid_volume) = (best_bid.0.to_owned(), best_bid.1.to_owned());
        if self.sequence > to_merge.sequence {
            // This happen in the beginning when older orderbook in websocket is received after REST
            return Err(std::format!(
                "[{}] -> [{}]",
                to_merge.sequence,
                self.sequence
            ));
        }
        // make sure that to_merge's PVMaps are already filtered such that
        // it is all behind the starting sequence
        self.sequence = to_merge.sequence;

        // merge BTreeMap with insert
        for (price, volume) in to_merge.ask.into_iter() {
            if volume.eq(&zero) {
                if self.ask.remove(&price).is_none() {
                    // log::error!("failed to remove ask at {}, no orderbook data", &price);
                }
                continue;
            }
            self.ask.insert(price, volume);
        }
        for (price, volume) in to_merge.bid.into_iter() {
            if volume.eq(&zero) {
                if self.bid.remove(&price).is_none() {
                    // log::error!("failed to remove bid at {}, no orderbook data", &price);
                }
                continue;
            }
            self.bid.insert(price, volume);
        }

        // if best ask exists in merge
        if let Some((&merge_min_ask, &merge_min_ask_volume)) = to_merge_clone.ask.first_key_value()
        {
            // either the merge has lower ask, or merge has same ask but increased in volume
            if merge_min_ask < min_ask
                || merge_min_ask == min_ask && merge_min_ask_volume > min_ask_volume
            {
                return Ok(Some(to_merge_clone));
            }
        }
        // if best bid exists in merge
        if let Some((&merge_max_bid, &merge_max_bid_volume)) = to_merge_clone.bid.last_key_value() {
            // either the merge has higher bid, or merge has same bid but increased in volume
            if merge_max_bid > max_bid
                || merge_max_bid == max_bid && merge_max_bid_volume > max_bid_volume
            {
                return Ok(Some(to_merge_clone));
            }
        }
        Ok(None)
    }
}
