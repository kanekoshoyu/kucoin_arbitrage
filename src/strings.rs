pub fn topic_to_symbol(topic: String) -> Option<String> {
    // from the websocket ticker topic
    let n = topic.find(":");
    if n.is_none() {
        return None;
    }
    let n = n.unwrap() + 1; //add 1 after ":"
    let x = topic.as_str();
    Some(String::from(&x[n..]))
}

pub fn symbol_to_tuple(ticker: &str) -> Option<(&str, &str)> {
    // regex to divide the tickers
    let n = ticker.find("-");
    if n.is_none() {
        return None;
    }
    let n = n.unwrap();
    Some(((&ticker[..n]), (&ticker[(n + 1)..])))
}
