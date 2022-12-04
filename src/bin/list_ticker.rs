extern crate kucoin_rs;

use kucoin_rs::failure;
use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};
use kucoin_rs::tokio::{self};
extern crate lazy_static;
use kucoin_arbitrage::tickers::bases_with_quotes;
use log::*;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    kucoin_arbitrage::shared::log_init();
    let api = Kucoin::new(KucoinEnv::Live, None)?;
    let res = bases_with_quotes(api, "BTC", "USDT").await?;
    let n = res.len();
    info!("Matched: {n}");
    // info!("res: {res:#?}");
    Ok(())
}
