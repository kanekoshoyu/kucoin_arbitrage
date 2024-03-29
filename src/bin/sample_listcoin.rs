use eyre::Result;
/// Lists pairs with the "BTC", "USDT" as quote
use kucoin_api::client::{Kucoin, KucoinEnv};
use kucoin_arbitrage::broker::symbol::{filter::symbol_with_quotes, kucoin::get_symbols};
#[tokio::main]
async fn main() -> Result<()> {
    // provide logging format
    // kucoin_arbitrage::logger::setup_logs(tracing::Leve::INFO, l)?;
    tracing::info!("Hello world");
    let config = kucoin_arbitrage::config::from_file("config.toml")?;
    let api = Kucoin::new(KucoinEnv::Live, Some(config.kucoin_credentials()))
        .map_err(|e| eyre::eyre!(e))?;

    // get symbol lists
    let symbol_list = get_symbols(api).await?;
    let res = symbol_with_quotes(&symbol_list, "BTC", "USDT");

    for r in res.clone().into_iter() {
        tracing::info!("{r:?}");
    }
    tracing::info!("size: {:?}", res.len());

    Ok(())
}
