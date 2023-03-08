use crate::{globals, strings};
use globals::orderbook::{self, get_local_asks, get_local_bids};
use kucoin_rs::kucoin::model::websocket::{Level2, WSResp};
use kucoin_rs::tokio::sync::broadcast;
use log::*;
use std::collections::HashMap;

static ERROR_PARSE_F64: &str = "Failed to parse value as f64";

// internally converts string into f64 and get the max key
pub fn get_max_key_ref(map: &HashMap<String, String>) -> (String, String) {
    let mut max_key = f64::MIN;
    let mut max_key_ref = &"".to_string();
    let mut val_ref = &"".to_string();
    for (key, value) in map {
        let key_f64: f64 = key.parse().expect(ERROR_PARSE_F64);
        if key_f64 > max_key {
            max_key = key_f64;
            max_key_ref = key;
            val_ref = value;
        }
    }
    // Only one conversion as below instead of every for loop round
    (max_key_ref.clone(), val_ref.clone())
}

// internally converts string into f64 and get the min key
pub fn get_min_key_ref(map: &HashMap<String, String>) -> (String, String) {
    let mut min_key = f64::MAX;
    let mut min_key_ref = &"".to_string();
    let mut val_ref = &"".to_string();
    for (key, value) in map {
        let key_f64: f64 = key.parse().expect(ERROR_PARSE_F64);
        if key_f64 < min_key {
            min_key = key_f64;
            min_key_ref = key;
            val_ref = value;
        }
    }
    (min_key_ref.clone(), val_ref.clone())
}

// get best ask/bid prices and volumes from the local order_book
pub fn get_best_ask_bid(symbol: &String) -> ((f64, f64), (f64, f64)) {
    let query_err = "query to symbol failed locally";
    // TODO: get the local ask/bid
    let local_asks = get_local_asks(symbol).expect(query_err);
    let local_bids = get_local_bids(symbol).expect(query_err);
    // TODO: get min ask, max bid
    let (best_ap, best_av) = get_min_key_ref(&local_asks.pricevolume);
    let (best_bp, best_bv) = get_max_key_ref(&local_bids.pricevolume);
    // TODO: convert to f64 and return
    return (
        (
            best_ap.parse::<f64>().unwrap(),
            best_av.parse::<f64>().unwrap(),
        ),
        (
            best_bp.parse::<f64>().unwrap(),
            best_bv.parse::<f64>().unwrap(),
        ),
    );
}

async fn check_arbitrage() {}

// This shuld go strategy
fn on_order_message_received(msg: WSResp<Level2>) {
    if msg.subject.ne("trade.l2update") {
        error!("unrecognised subject: {:?}", msg.subject);
        return;
    }
    // info!("received");
    // get the ticker name
    let ticker_name = strings::topic_to_symbol(msg.topic).expect("wrong ticker format");
    // info!("Ticker received: {ticker_name}");
    let data = msg.data;
    let symbol = data.symbol.clone();
    // info!("{:#?}", data);
    orderbook::store_orderbook_changes(&ticker_name, data);
    // do analysis with the symbol
    // TODO: shall we ditch the ticker_name and replace with the symbol?
    if symbol.eq("BTC-USDT") {
        return;
    }
    // check the arbitrage
}

// this function type is shared across different strategies
pub fn accept_(msg: WSResp<Level2>) {}

// this function type is shared across different strategies
// pub async fn get_receiver(msg: WSResp<Level2>) {
//     // get a receiver from BROADCAST, then make a copy 
//     let (tx, rx) = broadcast::channel(32);
//     let tx_clone = tx.clone();

//     task::spawn(async move {
//         for i in 0..10 {
//             tx_clone.send(i).await.unwrap();
//         }
//     });

//     // Spawn the subscriber tasks
//     let mut rx1 = rx.subscribe();
//     let mut rx2 = rx.subscribe();
//     task::spawn(async move {
//         while let Some(i) = rx1.recv().await {
//             println!("Subscriber 1 received: {}", i);
//         }
//     });
//     task::spawn(async move {
//         while let Some(i) = rx2.recv().await {
//             println!("Subscriber 2 received: {}", i);
//         }
//     });


// }
