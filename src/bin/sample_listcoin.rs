/// Lists pairs with the "BTC", "USDT" as quote
use kucoin_api::client::{Kucoin, KucoinEnv};
use kucoin_arbitrage::broker::symbol::{filter::symbol_with_quotes, kucoin::get_symbols};

#[tokio::main]
async fn main() -> Result<(), kucoin_api::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Hello world");

    // set credentials
    let credentials = kucoin_arbitrage::global::config::credentials();
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;

    // get symbol lists
    let symbol_list = get_symbols(api).await;
    let res = symbol_with_quotes(&symbol_list, "BTC", "USDT");

    for r in res.clone().into_iter() {
        log::info!("{r:?}");
    }
    log::info!("size: {:?}", res.len());

    Ok(())
}
