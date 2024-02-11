use crate::error::Error;
use core::str::FromStr;
use kucoin_api::client::Credentials;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Config {
    pub kucoin: KuCoinConfig,
    pub behaviour: BehaviourConfig,
    pub log: LogConfig,
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

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct KuCoinConfig {
    pub api_key: String,
    pub secret_key: String,
    pub passphrase: String,
}

impl From<&KuCoinConfig> for Credentials {
    fn from(config: &KuCoinConfig) -> Self {
        Credentials::new(&config.api_key, &config.secret_key, &config.passphrase)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BehaviourConfig {
    pub monitor_interval_sec: u32,
    pub usd_cyclic_arbitrage: u32,
}

pub fn from_file(filename: &str) -> Result<Config, Error> {
    let toml_str = std::fs::read_to_string(filename).map_err(Error::IoError)?;
    toml::from_str(&toml_str).map_err(Error::TomlError)
}

/// custom log level declared for the custom FromStr
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    #[default]
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for tracing::Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }
}

impl FromStr for LogLevel {
    type Err = eyre::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "error" => Ok(LogLevel::Error),
            "warn" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            _ => Err(eyre::eyre!("Invalid log level: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LogConfig {
    pub file_directory: String,
    pub file_prefix: String,
    pub file_log_level: LogLevel,
    pub term_log_level: LogLevel,
}

mod tests {
    #[test]
    fn load_config() {
        let toml_str = "
        [kucoin]
        api_key = \"YOUR_API_KEY_HERE\"
        secret_key = \"YOUR_SECRET_KEY_HERE\"
        passphrase = \"YOUR_PASSPHRASE_HERE\"
        [behaviour]
        monitor_interval_sec = 120
        usd_cyclic_arbitrage = 20
        [log]
        file_directory = \"./logs/\"
        file_prefix = \"log\"
        file_log_level = \"warn\"
        term_log_level = \"info\"
        ";
        let res = toml::from_str(toml_str);
        assert!(res.is_ok(), "malformed config");
        let config: super::Config = res.unwrap();
        assert_eq!(config.behaviour.monitor_interval_sec, 120);
        assert_eq!(config.behaviour.usd_cyclic_arbitrage, 20);

        assert_eq!(config.log.file_directory, "./logs/");
        assert_eq!(config.log.file_log_level, super::LogLevel::Warn);
        assert_eq!(config.log.term_log_level, super::LogLevel::Info);
    }
}
