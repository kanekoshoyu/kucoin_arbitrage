use crate::error::Error;
use kucoin_api::client::Credentials;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Config {
    pub kucoin: KuCoin,
    pub behaviour: Behaviour,
}

impl Config {
    pub fn kucoin_credentials(self) -> Credentials {
        Credentials::new(
            &self.kucoin.api_key,
            &self.kucoin.secret_key,
            &self.kucoin.passphrase,
        )
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KuCoin {
    pub api_key: String,
    pub secret_key: String,
    pub passphrase: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Behaviour {
    pub monitor_interval_sec: u32,
    pub usd_cyclic_arbitrage: u32,
}

pub fn from_file(filename: &str) -> Result<Config, Error> {
    let toml_str = std::fs::read_to_string(filename).map_err(Error::IoError)?;
    toml::from_str(&toml_str).map_err(Error::TomlError)
}
