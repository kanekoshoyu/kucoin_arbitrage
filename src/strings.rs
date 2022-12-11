// append "BTC" and "USDT" as "BTC-USDT"
pub fn symbol_string(base: &str, quote: &str) -> String {
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
