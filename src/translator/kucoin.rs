/*
    Translates from kucoin API into our model datatypes
*/

use crate::model::orderbook::{Orderbook, PVMap};
use crate::translator::translator;
use chrono;
use kucoin_rs::kucoin::model::{
    market::OrderBook as KucoinOrderBook, websocket::Level2 as KucoinOrderBookChange,
};
use ordered_float::OrderedFloat;

impl translator::OrderBookTranslator for KucoinOrderBook {
    fn to_internal(&self) -> Orderbook {
        let parse_err_msg = "Failed to parse input";
        let sequence = self.sequence.parse::<u64>().unwrap();
        let mut ask = PVMap::new();
        let mut bid = PVMap::new();

        for ask_pv in self.asks.clone() {
            let price: OrderedFloat<f64> = ask_pv[0].parse().expect(parse_err_msg);
            let volume: OrderedFloat<f64> = ask_pv[1].parse().expect(parse_err_msg);
            ask.insert(price, volume);
        }
        for bid_pv in self.bids.clone() {
            let price: OrderedFloat<f64> = bid_pv[0].parse().expect(parse_err_msg);
            let volume: OrderedFloat<f64> = bid_pv[1].parse().expect(parse_err_msg);
            bid.insert(price, volume);
        }
        Orderbook { ask, bid, sequence }
    }
}

impl translator::OrderBookChangeTranslator for KucoinOrderBookChange {
    fn to_internal(&self, serial: u64) -> (String, Orderbook) {
        // return Orderbook::new();
        let mut ask = PVMap::new();
        let mut bid = PVMap::new();
        let parse_err_msg = "Failed to parse input";

        for ask_change in self.changes.asks.clone() {
            // ignore if sequence <=serial
            if ask_change[2].parse::<u64>().unwrap() > serial {
                let price: OrderedFloat<f64> = ask_change[0].parse().expect(parse_err_msg);
                let volume: OrderedFloat<f64> = ask_change[1].parse().expect(parse_err_msg);
                ask.insert(price, volume);
            }
        }
        for bid_change in self.changes.bids.clone() {
            // ignore if sequence <=serial
            if bid_change[2].parse::<u64>().unwrap() > serial {
                let price: OrderedFloat<f64> = bid_change[0].parse().expect(parse_err_msg);
                let volume: OrderedFloat<f64> = bid_change[1].parse().expect(parse_err_msg);
                bid.insert(price, volume);
            }
        }
        let sequence = self.sequence_end.clone() as u64;
        (self.symbol.clone(), Orderbook { ask, bid, sequence })
    }
}
