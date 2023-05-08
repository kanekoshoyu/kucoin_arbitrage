/// Get the Symbol list, and blacklist a few weird looking ones
use kucoin_arbitrage::broker::symbol::kucoin::get_symbols;
use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};

#[tokio::main]
async fn main() -> Result<(), kucoin_rs::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Hello world");

    // set credentials
    let credentials = kucoin_arbitrage::globals::config::credentials();
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;

    // get the data
    let symbol_list = get_symbols(api).await;
    // debugging
    // for symbol in symbol_list{
    //     log::info!("{symbol:?}");
    // }
    log::info!("size: {:?}", symbol_list.len());

    return Ok(());
}
