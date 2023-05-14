use crate::model::symbol::SymbolInfo;
use crate::translator::translator::SymbolInfoTranslator;
use kucoin_api::client::Kucoin;
use kucoin_api::model::market::SymbolList;

/// Uses the KuCoin API to generate a list of symbols
pub async fn get_symbols(api: Kucoin) -> Vec<SymbolInfo> {
    // TODO check what are the option for market selection from the API documentation
    let api_result = api.get_symbol_list(None).await;
    let data: Vec<SymbolList> = api_result.unwrap().data.unwrap();
    let mut result: Vec<SymbolInfo> = Vec::new();
    for symbol in data {
        // check base currency. Kucoin updates symbol instead of name when the alias updates
        if false == symbol.symbol.starts_with(symbol.base_currency.as_str()) {
            log::warn!(
                "name and base doesnt match (symbol: {:10}, name: {:10}, base: {:5})",
                symbol.symbol,
                symbol.name,
                symbol.base_currency
            );
            continue;
        }
        // check quote currency
        if symbol.quote_currency != symbol.fee_currency {
            log::warn!(
                "quote isn't fee \nquote: {:?}\nfee: {:?}",
                symbol.quote_currency,
                symbol.fee_currency
            );
            continue;
        }
        if symbol.symbol != symbol.name {
            // log::warn!(
            //     "name and symbol doen't match (symbol: {:10}, name: {:10}, base: {:5})",
            //     symbol.symbol,
            //     symbol.name,
            //     symbol.base_currency,
            // );
        }
        let internal_symbol_info = symbol.to_internal();
        result.push(internal_symbol_info);
    }
    return result;
}
