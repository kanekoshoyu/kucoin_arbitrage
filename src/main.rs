extern crate kucoin_rs;

use kucoin_rs::failure;
use kucoin_rs::futures::TryStreamExt;
use kucoin_rs::kucoin::websocket::KucoinWebsocket;
use kucoin_rs::tokio::{
    self,
    sync::Mutex,
    time::{sleep, Duration},
};

use chrono::Local;
use env_logger::Builder;
use kucoin_rs::kucoin::{
    client::{Credentials, Kucoin, KucoinEnv},
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
};
use log::*;
use std::io::Write;

// use a config file to store these instead of hardcoding, therefore the raw data
#[derive(Debug)]
struct Config {
    interval_sec: u64,
    api_key: &'static str,
    secret_key: &'static str,
    passphrase: &'static str,
}

#[derive(Debug)]
struct Performance {
    data_count: u64,
}
static DEF_STR: &str = "XYZ";

static PERFORMANCE: Mutex<Performance> = Mutex::const_new(Performance { data_count: 0 });
static CONFIG: Mutex<Config> = Mutex::const_new(Config {
    interval_sec: 2,
    api_key: DEF_STR,
    secret_key: DEF_STR,
    passphrase: DEF_STR,
});

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // provide logging format
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}]: {}",
                Local::now().format("%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    info!("Hello world");
    //TODO: use a config file instead of hard coding as above
    let mut api_key = DEF_STR;
    let mut secret_key = DEF_STR;
    let mut passphrase = DEF_STR;

    {
        // This bracket creates a critical　section
        let config = &mut *CONFIG.lock().await;
        api_key = config.api_key;
        secret_key = config.secret_key;
        passphrase = config.passphrase;
    }
    // If credentials are needed, generate a new Credentials struct w/ the necessary keys
    let credentials = Credentials::new(api_key, secret_key, passphrase);
    // Initialize the Kucoin API struct
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    // Generate the dynamic Public or Private websocket url and endpoint from Kucoin
    // which includes a token required for connecting
    let url = api.get_socket_endpoint(WSType::Public).await?;
    // Initialize the websocket
    let mut ws = api.websocket();

    // Generate a Vec<WSTopic> of desired subs.
    // Note they need to be public or private depending on the url

    
    // TODO: auto generate the vector using config, first use BTC as MID, USDT as base
    let subs = vec![
        WSTopic::Ticker(vec!["ETH-BTC".to_string()]),
        WSTopic::Ticker(vec!["BTC-USDT".to_string()]),
        WSTopic::Ticker(vec!["ETH-USDT".to_string()]),
    ];
    // each Ticker is max 10 data points per second

    // Initalize your subscription and use await to unwrap the future
    ws.subscribe(url, subs).await?;

    info!("Async polling");
    //TODO: arbitrage performance analysis, such as arbitrage chance per minute

    // TODO: average data set performance in a task
    tokio::spawn(async move { poll_task(ws).await });

    loop {
        {
            let config = &mut *CONFIG.lock().await;
            sleep(Duration::from_secs(config.interval_sec)).await;
        }
        report_status().await.expect("report status error");
    }
}

async fn report_status() -> Result<(), failure::Error> {
    info!("reporting");
    let performance = &mut *PERFORMANCE.lock().await;
    let config = &mut *CONFIG.lock().await;
    let data_rate = performance.data_count / config.interval_sec;
    info!("Data rate: {data_rate:?} points/sec");
    // clear the data
    performance.data_count = 0;

    Ok(())
}

// TODO; store the data into a map that mirrors a ticker status
async fn poll_task(mut ws: KucoinWebsocket) -> Result<(), failure::Error> {
    while let Some(msg) = ws.try_next().await? {
        // add matches for multi-subscribed sockets handling

        match msg {
            KucoinWebsocketMsg::TickerMsg(_msg) => {
                // info!("Ticker");
                let mut performance = &mut *PERFORMANCE.lock().await;
                performance.data_count += 1;
                // TODO: fill in the data
                // info!("{:#?}", msg)
            }
            // KucoinWebsocketMsg::PongMsg(_msg) => {
            //     info!("Ping");
            // }
            // KucoinWebsocketMsg::WelcomeMsg(_msg) => {}
            _ => {
                // panic!("unexpected msgs received: {msg:?}")
            }
        }
    }
    Ok(())
}

// have a task to act on the arbitrage

// suggest a way to implement the arbitrage search. Acting within poll task is probably not a good idea
