/*
    Translates from kucoin API into our model datatypes
*/

use crate::model::orderbook::{Orderbook, PVMap};
use crate::translator::translator;
use chrono;
use kucoin_rs::kucoin::model::{
    market::OrderBook as KucoinOrderBook, websocket::Level2 as KucoinOrderBookChange,
};

impl translator::OrderBookTranslator for KucoinOrderBook {
    fn to_internal(&self) -> Orderbook {
        let sequence = self.sequence.parse::<u64>().unwrap();
        let time = self.time;
        let mut ask = PVMap::new();
        let mut bid = PVMap::new();
        for ask_pv in self.asks.clone() {
            ask.insert(ask_pv[0].clone(), ask_pv[1].clone());
        }
        for bid_pv in self.bids.clone() {
            bid.insert(bid_pv[0].clone(), bid_pv[1].clone());
        }
        Orderbook {
            ask,
            bid,
            time,
            sequence,
        }
    }
}

impl translator::OrderBookChangeTranslator for KucoinOrderBookChange {
    fn to_internal(&self, serial: u64) -> (String, Orderbook) {
        // return Orderbook::new();
        let mut ask = PVMap::new();
        let mut bid = PVMap::new();

        for ask_change in self.changes.asks.clone() {
            // ignore if sequence <=serial
            if ask_change[2].parse::<u64>().unwrap() > serial {
                ask.insert(ask_change[0].clone(), ask_change[1].clone());
            }
        }
        for bid_change in self.changes.bids.clone() {
            // ignore if sequence <=serial
            if bid_change[2].parse::<u64>().unwrap() > serial {
                bid.insert(bid_change[0].clone(), bid_change[1].clone());
            }
        }
        let sequence = self.sequence_end.clone() as u64;
        let time = chrono::offset::Utc::now().timestamp();
        (
            self.symbol.clone(),
            Orderbook {
                ask,
                bid,
                time,
                sequence,
            },
        )
    }
}