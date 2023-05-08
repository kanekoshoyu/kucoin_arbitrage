/// Get the Symbol list, and blacklist a few weird looking ones
use kucoin_arbitrage::translator::translator::SymbolInfoTranslator;
use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};
use kucoin_rs::kucoin::model::market::SymbolList;

#[tokio::main]
async fn main() -> Result<(), kucoin_rs::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    log::info!("Hello world");

    // set credentials
    let credentials = kucoin_arbitrage::globals::config::credentials();
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;

    // get the data
    let data = api.get_symbol_list(None).await.unwrap();
    let symbol_list: Vec<SymbolList> = data.data.unwrap();
    for symbol in symbol_list {
        if symbol.quote_currency != symbol.fee_currency {
            log::warn!(
                "quote isnt fee \nquote: {:?}\nfee: {:?}",
                symbol.quote_currency,
                symbol.fee_currency
            );
            continue;
        }

        let stat = symbol.name.find(symbol.base_currency.as_str());
        if stat.is_none() || stat.unwrap() != 0 {
            log::warn!(
                "name and base doesnt fit \nsymbol: {:?}\nname: {:?},\nbase: {:?}",
                symbol.symbol,
                symbol.name,
                symbol.base_currency
            );
            continue;
        }
        if symbol.symbol != symbol.name {
            log::warn!(
                "symbol and name doesnt match (but name matches base)\nsymbol: {:?}\nname: {:?}\nbase: {:?}",
                symbol.symbol,
                symbol.name,
                symbol.base_currency,
            );
        }
        let internal_symbol_info = symbol.to_internal();
        log::info!("created: {:?}", internal_symbol_info)
    }

    return Ok(());
}
