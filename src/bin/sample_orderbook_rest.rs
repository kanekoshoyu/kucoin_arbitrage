/// Gets Orderbook from REST API
use kucoin_api::client::{Kucoin, KucoinEnv};
use kucoin_api::model::market::OrderBookType;

#[tokio::main]
async fn main() -> Result<(), kucoin_api::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Hello world");

    // credentials
    let credentials = kucoin_arbitrage::global::config::credentials();
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;

    let symbol_name = "BTC-USDT";
    let res = api.get_orderbook(symbol_name, OrderBookType::L20).await?;
    if let Some(orderbook) = res.data {
        log::info!("{orderbook:#?}");
    } else {
        log::info!("failed obtaining the proper data")
    }

    Ok(())
}
