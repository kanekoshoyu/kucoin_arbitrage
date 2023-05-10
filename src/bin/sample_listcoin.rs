/// Get the Symbol list, and blacklist a few weird looking ones
use kucoin_arbitrage::broker::symbol::{filter::symbol_with_quotes, kucoin::get_symbols};
use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};

#[tokio::main]
async fn main() -> Result<(), kucoin_rs::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Hello world");

    // set credentials
    let credentials = kucoin_arbitrage::globals::config::credentials();
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;

    // get symbol lists
    let symbol_list = get_symbols(api).await;
    let res = symbol_with_quotes(&symbol_list, "BTC", "USDT");

    for r in res.clone().into_iter() {
        log::info!("{r:?}");
    }
    log::info!("size: {:?}", res.len());

    return Ok(());
}
