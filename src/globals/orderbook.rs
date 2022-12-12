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

    // TODO: update the bids/asks in orderbook
    let end = l2.sequence_end.to_string();
    let _asks = l2.changes.asks;
    let _bid = l2.changes.bids;

    // update orderbook time and sequence
    orderbook.sequence = end;
    orderbook.time = time;

    Ok(())
}

pub fn get_clone(symbol: String) -> Option<OrderBook> {
    // TODO: make it return none if it is actually none
    let mut p = ORDERMAP.lock().unwrap();
    let res = (*p).get_mut(&symbol).unwrap();
    let res = OrderBook {
        sequence: res.sequence.clone(),
        time: res.time,
        bids: res.bids.clone(),
        asks: res.asks.clone(),
    };
    Some(res)
}
