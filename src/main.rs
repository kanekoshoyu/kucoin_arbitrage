extern crate kucoin_rs;

use kucoin_rs::failure;
use kucoin_rs::futures::TryStreamExt;
use kucoin_rs::tokio;

use kucoin_rs::kucoin::client::{Credentials, Kucoin, KucoinEnv};
use kucoin_rs::kucoin::model::websocket::{KucoinWebsocketMsg, WSTopic, WSType};

use log::*;

use env_logger::Builder;

use chrono::Local;
use log::LevelFilter;
use std::io::Write;

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

    // If credentials are needed, generate a new Credentials struct w/ the necessary keys

    let credentials = Credentials::new(
        "xxxxxxxxxxxxxXXXXXXxxx",
        "XXxxxxx-xxxxxx-xXxxxx-xxxx",
        "xxxxxx",
    );

    // Initialize the Kucoin API struct
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;

    // Generate the dynamic Public or Private websocket url and endpoint from Kucoin
    // which includes a token required for connecting
    let url = api.get_socket_endpoint(WSType::Public).await?;

    // Initialize the websocket
    let mut ws = api.websocket();

    // Generate a Vec<WSTopic> of desired subs. Note they need to be public or private
    // depending on the url
    let subs = vec![
        WSTopic::Ticker(vec!["BTC-USDT".to_string()]),
        WSTopic::Ticker(vec!["ETH-USDT".to_string()]),
        WSTopic::Ticker(vec!["ETH-BTC".to_string()]),
    ];

    // Initalize your subscription and use await to unwrap the future
    ws.subscribe(url, subs).await?;

    // Handle incoming responses matching messages. Note, the message matching is
    // not required for a single subscription but may be desired
    // for more complex event handling for multi-subscribed sockets add the additional
    // KucoinWebSocketMsg matches.
    info!("Async polling");
    //TODO: system performance analysis, such as data point rate
    //TODO: barbitrage performance analysis, such as arbitrage chance per minute

    // TODO: average data set performance in a task

    while let Some(msg) = ws.try_next().await? {
        match msg {
            KucoinWebsocketMsg::TickerMsg(msg) => {
                info!("Ticker");
                info!("{:#?}", msg)
            }
            KucoinWebsocketMsg::PongMsg(msg) => {
                info!("Ping");
                info!("{:#?}", msg); // Optional
            }
            KucoinWebsocketMsg::WelcomeMsg(msg) => {
                info!("Welcome Messsage");
                info!("{:#?}", msg)
            } // Optional
            _ => {
                panic!("default error!")
            } // Optional,
        }
    }
    Ok(())
}
