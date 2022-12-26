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

use std::time::{SystemTime, UNIX_EPOCH};

pub fn print_changes(changes: Vec<Vec<String>>) {
    for change in changes.into_iter() {
        if change.len().ne(&3) {
            panic!("wrong format");
        }
        let price = change.get(0).unwrap();
        let size = change.get(1).unwrap();
        let sequence = change.get(2).unwrap();
        log::info!("price:{price}");
        log::info!("size:{size}");
        log::info!("sequence:{sequence}");
    }
}
pub fn print_l2(l2: Level2) {
    log::info!("print_l2");
    let asks = l2.changes.asks;
    let bids = l2.changes.bids;
    if asks.len() > 0 {
        log::info!("asks");
        print_changes(asks);
    }
    if bids.len() > 0 {
        log::info!("bids");
        print_changes(bids);
    }
}

pub fn update_ws(symbol: String, l2: Level2) -> Result<(), kucoin_rs::failure::Error> {
    let mut p = ORDERMAP.lock().unwrap();
    // get mutable reference to the specific ordebook for bids/asks
    print_l2(l2.clone());

    let mut orderbook = (*p).get_mut(&symbol).expect("symbol data not found");
    let local_sequence = orderbook.sequence.parse::<i64>().unwrap();

    if l2.sequence_end < local_sequence {
        log::error!("Sequence not aligned");
        return Ok(());
    }

    // TODO: update the bids/asks in orderbook
    let end = l2.sequence_end.to_string();
    let _asks = l2.changes.asks;
    let _bid = l2.changes.bids;

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    // update orderbook time and sequence
    orderbook.sequence = end;
    orderbook.time = since_the_epoch.as_millis() as i64;

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
