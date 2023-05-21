/*
    Translates from kucoin_api crates model to out internal model
*/

use crate::model as internal_model;
use crate::translator::traits;
use kucoin_api::model as kucoin_api_model;
use ordered_float::OrderedFloat;

impl traits::OrderBookTranslator for kucoin_api_model::market::OrderBook {
    fn to_internal(&self) -> internal_model::orderbook::Orderbook {
        let parse_err_msg = "Failed to parse input";
        let sequence = self.sequence.parse::<u64>().unwrap();
        let mut ask = internal_model::orderbook::PVMap::new();
        let mut bid = internal_model::orderbook::PVMap::new();

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
        internal_model::orderbook::Orderbook { ask, bid, sequence }
    }
}

impl traits::OrderBookChangeTranslator for kucoin_api_model::websocket::Level2 {
    fn to_internal(&self, last_serial: u64) -> (String, internal_model::orderbook::Orderbook) {
        // return Orderbook::new();
        let mut ask = internal_model::orderbook::PVMap::new();
        let mut bid = internal_model::orderbook::PVMap::new();
        let parse_err_msg = "Failed to parse input";

        for ask_change in self.changes.asks.clone() {
            // ignore if sequence <=serial
            if ask_change[2].parse::<u64>().unwrap() > last_serial {
                let price: OrderedFloat<f64> = ask_change[0].parse().expect(parse_err_msg);
                let volume: OrderedFloat<f64> = ask_change[1].parse().expect(parse_err_msg);
                ask.insert(price, volume);
            }
        }
        for bid_change in self.changes.bids.clone() {
            // ignore if sequence <=serial
            if bid_change[2].parse::<u64>().unwrap() > last_serial {
                let price: OrderedFloat<f64> = bid_change[0].parse().expect(parse_err_msg);
                let volume: OrderedFloat<f64> = bid_change[1].parse().expect(parse_err_msg);
                bid.insert(price, volume);
            }
        }
        let sequence = self.sequence_end as u64;
        (
            self.symbol.clone(),
            internal_model::orderbook::Orderbook { ask, bid, sequence },
        )
    }
}

impl traits::SymbolInfoTranslator for kucoin_api_model::market::SymbolList {
    fn to_internal(&self) -> internal_model::symbol::SymbolInfo {
        internal_model::symbol::SymbolInfo {
            symbol: self.symbol.clone(),
            base: self.base_currency.clone(),
            quote: self.quote_currency.clone(),
            base_increment: self.base_increment.parse().unwrap(),
            base_min: self.base_min_size.parse().unwrap(),
        }
    }
}
