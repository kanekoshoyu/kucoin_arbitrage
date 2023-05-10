/// get the Orderbook from REST API
use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};
use kucoin_rs::kucoin::model::market::OrderBookType;

#[tokio::main]
async fn main() -> Result<(), kucoin_rs::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Hello world");

    // credentials
    let credentials = kucoin_arbitrage::globals::config::credentials();
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;

    let symbol_name = "BTC-USDT";
    let res = api.get_orderbook(symbol_name, OrderBookType::L20).await?;
    if let Some(orderbook) = res.data {
        log::info!("{orderbook:#?}");
    } else {
        log::info!("failed obtaining the proper data")
    }

    return Ok(());
}
