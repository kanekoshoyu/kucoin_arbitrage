extern crate lazy_static;
use chrono::{DateTime, Utc};
use core::panic;
use kucoin_rs::kucoin::model::market::OrderBook; //http api
use kucoin_rs::kucoin::model::websocket::Level2;
//ws api
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/*
please refer to
https://docs.kucoin.com/#get-full-order-book-aggregated
https://docs.kucoin.com/#level-2-market-data
*/

// pub type OrderMap = HashMap<String, OrderBook>;
// lazy_static::lazy_static! {
//     pub static ref ORDERMAP: Arc<Mutex<OrderMap>> = Arc::new(Mutex::new(HashMap::new()));
// }

// PriceVolume is not Copy-able
pub type PriceVolumeMap = HashMap<String, String>; //Prices to Volume

#[derive(Debug, Clone)]
pub struct BookData {
    pub pricevolume: PriceVolumeMap,
    time: DateTime<Utc>,
    sequence: i64,
}
pub type PartialBook = HashMap<String, BookData>; //Symbols to BookData

lazy_static::lazy_static! {
    pub static ref ASKS: Arc<Mutex<PartialBook>> = Arc::new(Mutex::new(PartialBook::new()));
    pub static ref BIDS: Arc<Mutex<PartialBook>> = Arc::new(Mutex::new(PartialBook::new()));
    pub static ref BROADCAST: Arc<Mutex<(broadcast::Sender<(String, BookData)>, broadcast::Receiver<(String, BookData)>)>> = Arc::new(Mutex::new(broadcast::channel(32))); //Each entry of partialbook
}

// cannot use get(), since PartialBook is not copy-able
pub fn get_local_asks(symbol: &String) -> Option<BookData> {
    let mut p = ASKS.lock().unwrap();
    let res = (*p).remove(symbol).clone();
    return res;
}

pub fn get_local_bids(symbol: &String) -> Option<BookData> {
    let mut p = ASKS.lock().unwrap();
    let res = (*p).remove(symbol);
    return res;
}

pub fn set_local_asks(symbol: String, book_data: BookData) -> Option<BookData> {
    let mut p = ASKS.lock().unwrap();
    (*p).insert(symbol, book_data)
}

pub fn set_local_bids(symbol: String, book_data: BookData) -> Option<BookData> {
    let mut p = BIDS.lock().unwrap();
    (*p).insert(symbol, book_data)
}

// directly pass the order_book data obtained frtom the REST API (OrderBook)
// meant to run only for the first insertion
// return None when new value added
// return Some when there was a value beforehand
pub fn store_orderbook(symbol: String, orderbook: OrderBook) {
    let asks = orderbook.asks;
    let bids = orderbook.bids;
    let queue_err_msg = "symbol not found locally";

    //  TODO: Create new BookData here and store them
    // unimplemented!("implement local asks/bids creation ");
    let mut local_asks = BookData {
        pricevolume: PriceVolumeMap::new(),
        time: Utc::now(),
        sequence: 0,
    };
    let mut local_bids = BookData {
        pricevolume: PriceVolumeMap::new(),
        time: Utc::now(),
        sequence: 0,
    };

    let f1 = store_book_changes(&mut local_asks, asks).is_none();
    let f2 = store_book_changes(&mut local_bids, bids).is_none();
    if f1 || f2 {
        panic!("There were previous data stored locally");
    }
}

// pub fn get_orderbook(symbol: String) -> Option<&'static OrderBook> {
//     let mut p = ORDERMAP.lock().unwrap();
//     (*p).get(&symbol)
// }

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

// abstracted out from the globals, return None when stored first time, else return Some(sequence)
fn store_book_changes(book_data: &mut BookData, changes: Vec<Vec<String>>) -> Option<i64> {
    let mut updated_sequence: i64 = book_data.sequence;
    let format_err_msg = "wrong format";
    let mut is_none = true;
    for change in changes.into_iter() {
        if change.len().ne(&3) {
            panic!("{format_err_msg}");
        }
        let price = change.get(0).expect(format_err_msg);
        let size = change.get(1).expect(format_err_msg);
        let sequence = change.get(2).expect(format_err_msg);
        let sequence_int = sequence.parse::<i64>().expect(format_err_msg);
        // skip when the sequence is lower than current one
        // TODO: check with the official API Documentatiuon
        if sequence_int <= updated_sequence {
            continue;
        }
        // value higher than the sequnce
        updated_sequence = sequence_int;
        // if first time, None is stored
        if book_data
            .pricevolume
            .insert(price.clone(), size.clone())
            .is_some()
        {
            is_none = false;
        }
    }
    // update to the latest sequence
    book_data.sequence = updated_sequence;
    return if is_none {
        None
    } else {
        Some(updated_sequence)
    };
}

// compare two String storing f64
pub fn ge_string_as_f64() {
    unimplemented!()
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

// directly pass the order_book change data obtained from the websocket API (L2)
pub fn store_orderbook_changes(symbol: &String, l2: Level2) {
    // let mut p = ORDERMAP.lock().unwrap();
    // get mutable reference to the specific ordebook for bids/asks
    let queue_err_msg = "symbol not found locally";
    let mut local_asks = get_local_asks(symbol).expect(queue_err_msg);
    let mut local_bids = get_local_bids(symbol).expect(queue_err_msg);
    // print_l2(l2.clone());

    // let mut orderbook = (*p).get_mut(&symbol).expect("update_ws could not direct to symbol in local book");
    // let local_sequence = orderbook.sequence.parse::<i64>().unwrap();

    store_book_changes(&mut local_asks, l2.changes.asks);
    store_book_changes(&mut local_bids, l2.changes.bids);
}
