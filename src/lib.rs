/// Bridge between API calls and internal data structures, with tasks, functions and structs
pub mod broker;
/// Config TOML file reader
pub mod config;
/// Custom error
pub mod error;
/// Event (enums)
pub mod event;
/// Logger intialization
pub mod logger;
/// API independent model struct for both system and multi-exchange support
pub mod model;
/// MPS counter and globally-mapped string-timers for system monitoring
pub mod monitor;
/// Arbitrage strategy algorithms
pub mod strategy;
/// String functions
pub mod strings;
/// Traits/impl to convert between API crate models and internal models
pub mod translator;
