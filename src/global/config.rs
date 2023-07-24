use crate::model::config::Config;
use ini::{Ini, Properties};
use kucoin_api::client::Credentials;
use std::sync::Arc;

lazy_static::lazy_static! {
    static ref INI: Ini = Ini::load_from_file("config.ini").expect("config file not found");
    pub static ref SEC_CRED: Properties = INI.section(Some("KuCoin Credentials")).unwrap().clone();
    pub static ref SEC_BEHV: Properties = INI.section(Some("Behaviour")).unwrap().clone();
    pub static ref CONFIG: Arc<Config> = Arc::new(load_ini());
}

// might require macro to load the filename
pub fn load_ini() -> Config {
    let str_interval = SEC_BEHV.get("monitor_interval_sec").unwrap();
    let str_budget = SEC_BEHV.get("usd_cyclic_arbitrage").unwrap();
    Config {
        monitor_interval_sec: str_interval.parse::<u64>().unwrap(),
        usd_cyclic_arbitrage: str_budget.parse::<u64>().unwrap(),
        api_key: SEC_CRED.get("api_key").unwrap(),
        secret_key: SEC_CRED.get("secret_key").unwrap(),
        passphrase: SEC_CRED.get("passphrase").unwrap(),
    }
}

pub fn credentials() -> Credentials {
    Credentials::new(CONFIG.api_key, CONFIG.secret_key, CONFIG.passphrase)
}
