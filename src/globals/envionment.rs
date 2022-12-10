extern crate lazy_static;
use ini::{Ini, Properties};

// gets the jobs done
lazy_static::lazy_static! {
    static ref INI: Ini = Ini::load_from_file("config.ini").expect("config file not found");
    pub static ref SEC_CRED: Properties = INI.section(Some("Credentials")).unwrap().clone();
    pub static ref SEC_BEHV: Properties = INI.section(Some("Behaviour")).unwrap().clone();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ticker_read() {
        let topic = "/market/ticker:ETH-BTC";
        let wanted = "ETH-BTC";
        let n = topic.find(":");
        if n.is_none() {
            panic!(": not found");
        }
        let n = n.unwrap() + 1; //add 1 after ":"
        let slice = &topic[n..];
        assert_eq!(wanted, slice);
    }

    #[test]
    fn test_get_ticker_string() {
        let topic = String::from("/market/ticker:ETH-BTC");
        let wanted = "ETH-BTC";
        let slice = crate::strings::topic_to_symbol(topic).unwrap();
        println!("slice: {slice:?}");
        assert_eq!(wanted, slice);
    }

    #[test]
    fn test_symbol_to_tuple() {
        let topic = "ETH-BTC";
        let slice = crate::strings::symbol_to_tuple(topic);
        let slice = slice.expect("wrong format");
        println!("slice: {slice:?}");
        assert_eq!(slice, ("ETH", "BTC"));
    }
}
