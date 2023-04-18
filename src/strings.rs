// append "BTC" and "USDT" as "BTC-USDT"
pub fn symbol_to_string(base: &str, quote: &str) -> String {
    let mut n = String::from(base);
    n.push('-');
    n.push_str(quote);
    return n;
}

// get "BTC-USDT" from websocket topic name
pub fn topic_to_symbol(topic: String) -> Option<String> {
    let n = topic.find(":")? + 1;
    let x = topic.as_str();
    Some(String::from(&x[n..]))
}

// turn "BTC-USDT" into ("BTC", "USDT")
pub fn symbol_to_tuple(ticker: &str) -> Option<(&str, &str)> {
    let n = ticker.find("-")?;
    Some(((&ticker[..n]), (&ticker[(n + 1)..])))
}

/// Split a symbol like "BTC-USDT" into ("BTC", "USDT")
/// ```
/// use kucoin_arbitrage::strings::split_symbol;
/// let res = split_symbol(String::from("BTC-USDT"));
/// assert_eq!(res.unwrap(), (String::from("BTC"), String::from("USDT")));
/// ```
pub fn split_symbol(symbol: String) -> Option<(String, String)> {
    let delimiter = "-";
    let substrings: Vec<String> = symbol
        .split(delimiter)
        .map(|s| s.trim().to_string())
        .collect();
    Some((substrings[0].clone(), substrings[1].clone()))
}
