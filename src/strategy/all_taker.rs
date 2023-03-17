use crate::globals;
use globals::legacy::orderbook::{get_local_asks, get_local_bids};
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
