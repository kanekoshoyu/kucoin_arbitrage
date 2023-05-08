/// get the Orderbook from REST API

use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};
use kucoin_rs::kucoin::model::market::OrderBookType;
use kucoin_rs::tokio;

use log::info;

#[tokio::main]
async fn main() -> Result<(), kucoin_rs::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    info!("Hello world");
    let credentials = kucoin_arbitrage::globals::config::credentials();
    info!("{credentials:#?}");
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;

    let symbol_name = "BTC-USDT";
    let ob_type = OrderBookType::L20;
    let res = api.get_orderbook(symbol_name, ob_type).await?;
    if let Some(orderbook) = res.data {
        info!("{orderbook:#?}");
    } else {
        info!("failed obtaining hte proper data")
    }

    return Ok(());
}
