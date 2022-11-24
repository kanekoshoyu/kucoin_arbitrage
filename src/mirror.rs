use chrono::{DateTime, Utc};
use kucoin_rs::kucoin::model::websocket::SymbolTicker;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type Map = HashMap<String, TickerInfo>;
lazy_static! {
    pub static ref MIRROR: Arc<Mutex<Map>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Clone)]
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
        TickerInfo {
            symbol,
            last_update: chrono::offset::Utc::now(),
        }
    }
}

impl TickerInfo {
    pub fn new(symbol: SymbolTicker) -> Self {
        TickerInfo {
            symbol,
            last_update: chrono::offset::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    #[test]
    fn test_insert_and_read() {
        let ticker_name = "BTC-USDT".to_string();
        let def_ticker = crate::mirror::TickerInfo::default();
        let ticker_name_clone = ticker_name.clone();

        let mir = crate::mirror::MIRROR.clone();
        {
            let mut mir = mir.lock().unwrap();
            let mir = &mut (*mir);
            let cloned = def_ticker.clone();
            mir.insert(ticker_name, cloned);
        }
        {
            let mut mir = mir.lock().unwrap();
            let mir = &mut (*mir);
            if let Some(_data) = mir.get_mut(&ticker_name_clone) {
                return; //value was inserted
            } else {
                panic!("not inserted");
            }
        }
    }
}
