extern crate lazy_static;
use super::envionment::*;
use kucoin_rs::kucoin::client::Credentials;
use std::sync::Arc;

#[derive(Debug, Default, Clone, Copy)]
pub struct Config {
    pub monitor_interval_sec: u64,
    pub api_key: &'static str,
    pub secret_key: &'static str,
    pub passphrase: &'static str,
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Arc<Config> = Arc::new(load_ini());
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

pub fn credentials() -> Credentials {
    Credentials::new(CONFIG.api_key, CONFIG.secret_key, CONFIG.passphrase)
}
