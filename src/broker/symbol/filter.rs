use std::collections::BTreeMap;

use crate::model::symbol::SymbolInfo;

/// filter the symbol list with a quote currency
pub fn symbol_with_quote(symbols: &Vec<SymbolInfo>, quote: &str) -> Vec<SymbolInfo> {
    let mut result: Vec<SymbolInfo> = Vec::new();
    for symbol in symbols {
        if symbol.quote == quote {
            result.push(symbol.clone());
        }
    }
    result
}

/// Filter the symbol list with a base that has both BTC and USDT as the quote currency
pub fn symbol_with_quotes(symbols: &Vec<SymbolInfo>, btc: &str, usd: &str) -> Vec<SymbolInfo> {
    let mut base_map: BTreeMap<String, (Option<SymbolInfo>, Option<SymbolInfo>)> = BTreeMap::new();

    for symbol in symbols {
        // check symbol.quote
        if symbol.quote != *btc && symbol.quote != *usd {
            continue;
        }
        let entry = base_map.entry(symbol.base.clone()).or_insert((None, None));
        if symbol.quote == *btc {
            // btc
            entry.0 = Some(symbol.clone());
        } else if symbol.quote == *usd {
            // usd
            entry.1 = Some(symbol.clone());
        }
    }

    let mut result = Vec::new();
    for (_, entry) in base_map {
        if let (Some(btc_symbol), Some(usd_symbol)) = entry {
            result.push(btc_symbol);
            result.push(usd_symbol);
        } else if let Some(usd_symbol) = entry.1 {
            // special case for btc-usd
            if usd_symbol.base == btc {
                result.insert(0, usd_symbol);
            }
        }
    }
    result
}

/// convert the vector into BTreeMap
pub fn vector_to_hash(symbols: &Vec<SymbolInfo>) -> BTreeMap<String, SymbolInfo> {
    let mut result: BTreeMap<String, SymbolInfo> = BTreeMap::new();
    for symbol in symbols {
        result.insert(symbol.symbol.clone(), symbol.clone());
    }
    result
}
