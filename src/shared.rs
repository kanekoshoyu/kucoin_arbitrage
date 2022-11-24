extern crate lazy_static;
use ini::{Ini, Properties};
use lazy_static::lazy_static;

#[derive(Debug, Default, Clone, Copy)]
pub struct Config {
    pub monitor_interval_sec: u64,
    pub api_key: &'static str,
    pub secret_key: &'static str,
    pub passphrase: &'static str,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Performance {
    pub data_count: u64,
}

// gets the jobs done
lazy_static! {
    pub static ref INI: Ini = Ini::load_from_file("config.ini").expect("config file not found");
    pub static ref SEC_CRED: Properties = INI.section(Some("Credentials")).unwrap().clone();
    pub static ref SEC_BEHV: Properties = INI.section(Some("Behaviour")).unwrap().clone();
}

// might require macro to load the filename
pub fn load_ini() -> Config {
    let interval_str = SEC_BEHV.get("monitor_interval_sec").unwrap();
    Config {
        monitor_interval_sec: interval_str.parse::<u64>().unwrap(),
        api_key: SEC_CRED.get("api_key").unwrap(),
        secret_key: SEC_CRED.get("secret_key").unwrap(),
        passphrase: SEC_CRED.get("passphrase").unwrap(),
    }
}

use env_logger::Builder;
use std::io::Write;

pub fn log_init() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}]: {}",
                chrono::Local::now().format("%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, log::LevelFilter::Info)
        .init();
}

pub fn topic_to_ticker(topic: String) -> Option<String> {
    // from the websocket ticker topic
    let n = topic.find(":");
    if n.is_none() {
        return None;
    }
    let n = n.unwrap() + 1; //add 1 after ":"
    let x = topic.as_str();
    let x = &x[n..];
    let x = String::from(x);
    Some(x)
}

pub fn ticker_to_tuple(text: String) -> Option<(String, String)> {
    // regex to divide the tickers
    let res = text.as_str();
    if res.find("-").is_none() {
        return None;
    }
    let n = res.find("-").unwrap();
    Some(((&res[..n]).to_string(), (&res[(n + 1)..]).to_string()))
    // let r2 = &res[(n + 1)..];

    // let splitter = Regex::new(r"[-]").unwrap();
    // // info!("{text}");
    // let txt: &str = &text[..];
    // let splits: Vec<_> = splitter.split(txt).into_iter().collect();
    // if 2 != splits.len() {
    //     panic!("invalid format {splits:?}")
    // }
    // (splits[0].to_string(), splits[1].to_string())
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
        let slice = crate::shared::topic_to_ticker(topic).unwrap();
        println!("slice: {slice:?}");
        assert_eq!(wanted, slice);
    }

    #[test]
    fn test_symbol_to_tuple() {
        let topic = String::from("ETH-BTC");
        let slice = crate::shared::ticker_to_tuple(topic);
        let slice = slice.expect("wrong format");
        println!("slice: {slice:?}");
        assert_eq!(slice, (String::from("ETH"), String::from("BTC")));
    }
}
