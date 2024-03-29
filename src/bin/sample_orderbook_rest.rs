use eyre::Result;
/// Gets Orderbook from REST API
use kucoin_api::client::{Kucoin, KucoinEnv};
use kucoin_api::model::market::OrderBookType;
#[tokio::main]
async fn main() -> Result<()> {
    // provide logging format
    // kucoin_arbitrage::logger::log_init()?;
    tracing::info!("Hello world");

    let config = kucoin_arbitrage::config::from_file("config.toml")?;
    let api = Kucoin::new(KucoinEnv::Live, Some(config.kucoin_credentials()))
        .map_err(|e| eyre::eyre!(e))?;

    let symbol_name = "BTC-USDT";
    let res = api
        .get_orderbook(symbol_name, OrderBookType::L20)
        .await
        .map_err(|e| eyre::eyre!(e))?;
    let orderbook = res.data.expect("failed obtaining the proper data");
    tracing::info!("{orderbook:#?}");
    Ok(())
}
