extern crate kucoin_rs;

use kucoin_rs::failure;
use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};
use kucoin_rs::kucoin::model::market::SymbolList;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Hello world");
    let credentials = kucoin_arbitrage::globals::config::credentials();
    log::info!("{credentials:#?}");
    // Initialize the Kucoin API struct
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;

    //  get all the data from symbol first, then obtain the symbil info
    let result = api
        .get_symbol_list(None)
        .await
        .expect("failed getting symbol list");
    if let Some(mut data) = result.data {
        let n = data.len();

        for datum in data.iter_mut() {
            let x: &SymbolList = datum;
            let name = &x.name;
            log::info!("{name:#?}");
            // log::info!("result: {datum:#?}");
        }
        log::info!("total symbols: {n:?}")
    }
    return Ok(());
}
