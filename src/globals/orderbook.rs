extern crate lazy_static;
use kucoin_rs::kucoin::model::market::OrderBook; //http api
use kucoin_rs::kucoin::model::websocket::Level2; //ws api
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/*
please refer to
https://docs.kucoin.com/#get-full-order-book-aggregated
https://docs.kucoin.com/#level-2-market-data
*/

pub type OrderMap = HashMap<String, OrderBook>;
lazy_static::lazy_static! {
    pub static ref ORDERMAP: Arc<Mutex<OrderMap>> = Arc::new(Mutex::new(HashMap::new()));
}

// return None when new value added
// return Some when there was a value beforehand
pub fn insert_book(symbol: String, orderbook: OrderBook) -> Option<OrderBook> {
    let mut p = ORDERMAP.lock().unwrap();
    (*p).insert(symbol, orderbook)
}

pub fn update_ws(symbol: String, l2: Level2, time: i64) -> Result<(), kucoin_rs::failure::Error> {
    let mut p = ORDERMAP.lock().unwrap();
    // get mutable reference to the specific ordebook for bids/asks
    let mut orderbook = (*p).get_mut(&symbol).unwrap();

    let end = l2.sequence_end.to_string();
    let asks = l2.changes.asks;
    let bid = l2.changes.bids;
    // update the bids/asks in orderbook
    // orderbook
    // update orderbook time and sequence
    orderbook.sequence = end;
    orderbook.time = time;

    Ok(())
}

pub fn get_clone(symbol: String) -> Option<OrderBook> {
    // TODO: make it return none if it is actually none
    let mut p = ORDERMAP.lock().unwrap();
    let x = (*p).get_mut(&symbol).unwrap();
    let x = OrderBook {
        sequence: x.sequence.clone(),
        time: x.time,
        bids: x.bids.clone(),
        asks: x.asks.clone(),
    };
    Some(x)
}

#[cfg(test)]
mod tests {
    use crate::globals::orderbook;
    use core::panic;

    #[test]
    fn test_l2_format() {
        let symbols = ["BTC-USDT", "ETH-USDT", "ETH-USDT"];

        // TODO: finish
        unimplemented!();
        // let mir = orderbook::insert_book();

    }
}
