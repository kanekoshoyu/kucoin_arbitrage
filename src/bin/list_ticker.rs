extern crate kucoin_rs;

use kucoin_rs::failure;
use kucoin_rs::kucoin::model::market::Tick;
use kucoin_rs::tokio::{self};

use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};

// provide eazy data
extern crate lazy_static;

use log::*;
use std::collections::HashMap;

use std::str::FromStr;

use kucoin_arbitrage::shared::*;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    kucoin_arbitrage::shared::log_init();
    let api = Kucoin::new(KucoinEnv::Live, None)?;
    let res = api.get_all_tickers().await?;
    info!("Hello world");

    let all_ticker = res.data.expect("connection failure");
    info!("Time: {:#?}", all_ticker.time);
    let tickers = all_ticker.ticker;
    let total = tickers.len();
    info!("Total: {:#?}", total); //1299

    let mut dict: HashMap<String, bool> = HashMap::new();

    let mut n = 0;
    for ticker in tickers.into_iter() {
        let t: Tick = ticker;
        let symbol = t.symbol;
        let (a, b) = ticker_to_tuple(symbol).expect("wrong format");
        // info!("{a}:{b}");
        if quote_is_match(&mut dict, &a, &b) {
            // ticker a is the one
            info!("{:?}", a);
            n += 1;
        }
    }
    info!("Matched: {n}");
    Ok(())
}

/*
    Symbol looks like this
    "GMB-BTC"
    "TRIBE-USDT"
    "MLK-USDT"
*/

// rough way of reading.
fn quote_is_match(dict: &mut HashMap<String, bool>, quote: &String, base: &String) -> bool {
    // no match
    let base1 = "BTC";
    let base2 = "USDT";
    if base.ne(base1) && base.ne(base2) {
        return false;
    }
    // first match
    if dict.get(quote).is_none() {
        let quote_str = quote.as_str();
        let quote = String::from_str(quote_str).unwrap();
        dict.insert(quote, false);
        return false;
    }
    // second match
    *(dict.get_mut(quote).unwrap()) = true;
    return true;
}
