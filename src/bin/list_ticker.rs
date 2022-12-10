extern crate kucoin_rs;

use kucoin_rs::failure;
use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};
use kucoin_rs::tokio::{self};
use kucoin_arbitrage::tickers::bases_with_quotes;
use log::*;
use kucoin_arbitrage::logger;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    logger::log_init();
    let api = Kucoin::new(KucoinEnv::Live, None)?;
    let res = bases_with_quotes(api, "BTC", "USDT").await?;
    let n = res.len();
    info!("Matched: {n}");
    // info!("res: {res:#?}");
    Ok(())
}
