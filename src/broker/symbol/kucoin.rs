use crate::model::symbol::SymbolInfo;
use crate::translator::traits::SymbolInfoTranslator;
use kucoin_api::client::Kucoin;
use kucoin_api::model::market::SymbolList;
use kucoin_api::model::websocket::WSTopic;

/// Uses the KuCoin API to generate a list of symbols
pub async fn get_symbols(api: Kucoin) -> Vec<SymbolInfo> {
    // TODO check what are the option for market selection from the API documentation
    let api_result = api.get_symbol_list(None).await;
    let data: Vec<SymbolList> = api_result.unwrap().data.unwrap();
    let mut result: Vec<SymbolInfo> = Vec::new();
    for symbol in data {
        // check base currency. Kucoin updates symbol instead of name when the alias updates
        if !symbol.symbol.starts_with(symbol.base_currency.as_str()) {
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
    result
}

// TODO this bridges between API and the internal model, it should be placed in broker
pub fn format_subscription_list(infos: &[SymbolInfo]) -> Vec<Vec<WSTopic>> {
    // Extracts the symbol name from SynbolInfo
    let symbols: Vec<String> = infos.iter().map(|info| info.symbol.clone()).collect();

    // Sets up 2D array of max length 100
    let max_sub_count = 100;
    let mut hundred_arrays: Vec<Vec<String>> = Vec::new();
    let mut hundred_array: Vec<String> = Vec::new();

    // feed into the 2D array
    for symbol in symbols {
        hundred_array.push(symbol);
        // 99 for the first one, because of the special BTC-USDT
        if hundred_arrays.is_empty() && hundred_array.len() == max_sub_count - 1 {
            hundred_arrays.push(hundred_array);
            hundred_array = Vec::new();
            continue;
        }
        // otherwise 100
        if hundred_array.len() == max_sub_count {
            hundred_arrays.push(hundred_array);
            hundred_array = Vec::new();
        }
    }

    // last array in current_subarray
    if !hundred_array.is_empty() {
        hundred_arrays.push(hundred_array);
    }

    let mut subs: Vec<Vec<WSTopic>> = Vec::new();
    let mut sub: Vec<WSTopic> = Vec::new();
    for sub_array in hundred_arrays {
        sub.push(WSTopic::OrderBook(sub_array));
        if sub.len() == 3 {
            subs.push(sub);
            sub = Vec::new();
        }
    }
    subs
}
