use chrono::{DateTime, Utc};
use kucoin_api::model::websocket::SymbolTicker;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type Map = HashMap<String, TickerInfo>;
lazy_static! {
    pub static ref MIRROR: Arc<Mutex<Map>> = Arc::new(Mutex::new(HashMap::new()));
}

pub fn insert(ticker_name: String, ticker_info: TickerInfo) {
    let mut mir = MIRROR.lock().unwrap();
    (*mir).insert(ticker_name, ticker_info);
}

pub fn has(ticker_name: String) -> bool {
    let mut mir = MIRROR.lock().unwrap();
    (*mir).get_mut(&ticker_name).is_some()
}

#[derive(Debug, Clone)]
pub struct TickerInfo {
    pub symbol: SymbolTicker,
    pub last_update: DateTime<Utc>,
}

impl Default for TickerInfo {
    fn default() -> TickerInfo {
        let symbol = SymbolTicker {
            sequence: String::default(),
            best_ask: String::default(),
            size: String::default(),
            best_bid_size: String::default(),
            price: String::default(),
            best_ask_size: String::default(),
            best_bid: String::default(),
        };
        TickerInfo::new(symbol)
    }
}

impl TickerInfo {
    pub fn new(symbol: SymbolTicker) -> Self {
        TickerInfo {
            symbol,
            last_update: chrono::offset::Utc::now(),
        }
    }

    // return price and size
    pub fn get_bid(&self) -> (f64, f64) {
        let float_err = "float deparse error";
        let p = self.symbol.best_bid.parse::<f64>().expect(float_err);
        let v = self.symbol.best_bid_size.parse::<f64>().expect(float_err);
        (p, v)
    }

    // return price and size
    pub fn get_ask(&self) -> (f64, f64) {
        let float_err = "float deparse error";
        let p = self.symbol.best_ask.parse::<f64>().expect(float_err);
        let v = self.symbol.best_ask_size.parse::<f64>().expect(float_err);
        (p, v)
    }

    // merged so no copy twice
    pub fn get_askbid(&self) -> ((f64, f64), (f64, f64)) {
        let ask = self.get_ask();
        let bid = self.get_bid();
        (ask, bid)
    }
}

#[cfg(test)]
mod tests {
    use crate::mirror;
    use core::panic;

    #[test]
    fn test_insert_and_read() {
        let ticker_name = "BTC-USDT".to_string();
        let def_ticker = mirror::TickerInfo::default();
        mirror::insert(ticker_name.clone(), def_ticker.clone());
        if !mirror::has(ticker_name) {
            panic!("not inserted");
        }
    }
}
