/*
    lazy statics: ORDERBOOK
*/
extern crate lazy_static;
use crate::model::orderbook::FullOrderbook;
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    pub static ref ORDERBOOK: Arc<Mutex<FullOrderbook>> = Arc::new(
        Mutex::new(FullOrderbook::new()));
}
