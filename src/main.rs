extern crate kucoin_rs;

use kucoin_rs::failure;
use kucoin_rs::futures::TryStreamExt;
use kucoin_rs::kucoin::websocket::KucoinWebsocket;
use kucoin_rs::tokio::{
    self,
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
use std::sync::{Arc, Mutex};

// custom shared structs
mod shared;
use shared::*;
// provide eazy data
extern crate lazy_static;
use lazy_static::lazy_static;

// gets the jobs done
lazy_static! {
    static ref CONFIG: Arc<Mutex<Config>> = Arc::new(Mutex::new(load_ini()));
    static ref PERFORMANCE: Arc<Mutex<Performance>> =
        Arc::new(Mutex::new(Performance { data_count: 0 }));
}

// Arc has implicit 'static bound, so it cannot contain reference to local variable.
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

    let c = CONFIG.clone();
    let mg = c.lock().unwrap();
    let credentials = Credentials::new((*mg).api_key, (*mg).secret_key, (*mg).passphrase);
    drop(mg);

    info!("{credentials:#?}");
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
    let subs = vec![WSTopic::Ticker(vec![
        "ETH-BTC".to_string(),
        "BTC-USDT".to_string(),
        "ETH-USDT".to_string(),
    ])];
    /*
        each ticker is approx max 10 data points per second,
        its okay if there is duplicate in the pairs, but we can use a hashmap to manage the pairs

    */

    // Initalize your subscription and use await to unwrap the future
    ws.subscribe(url, subs).await?;

    info!("Async polling");
    // TODO: arbitrage performance analysis, such as arbitrage chance per minute

    let p = PERFORMANCE.clone();
    tokio::spawn(async move { poll_task(ws, p).await });

    let monitor_delay = {
        let c = CONFIG.clone();
        let mg = c.lock().unwrap();
        let interval_sec: u64 = (*mg).monitor_interval_sec;
        drop(mg);
        Duration::from_secs(interval_sec)
    };
    // main loop is for monitoring the system performance
    loop {
        sleep(monitor_delay).await;
        report_status(PERFORMANCE.clone(), CONFIG.clone()).expect("report status error");
    }
}

// Though PERFORMANCE and CONFIG are globally accessible at the moment, we need to clone it annyways. We can just clone in the main function
fn report_status(
    perf: Arc<Mutex<Performance>>,
    conf: Arc<Mutex<Config>>,
) -> Result<(), failure::Error> {
    info!("reporting");
    let p = perf.lock().unwrap();
    let c = conf.lock().unwrap();
    let data_rate = (*p).data_count / (*c).monitor_interval_sec;
    drop(p);
    drop(c);

    info!("Data rate: {data_rate:?} points/sec");
    // clear the data
    {
        let mut p = perf.lock().unwrap();
        (*p).data_count = 0;
    }

    Ok(())
}

// TODO; store the data into a map that mirrors a ticker status
async fn poll_task(
    mut ws: KucoinWebsocket,
    perf: Arc<Mutex<Performance>>,
) -> Result<(), failure::Error> {
    while let Some(msg) = ws.try_next().await? {
        // add matches for multi-subscribed sockets handling

        match msg {
            KucoinWebsocketMsg::TickerMsg(_msg) => {
                // info!("Ticker");
                // TODO: fill in the data
                // info!("{:#?}", msg)
                {
                    let mut p = perf.lock().unwrap();
                    (*p).data_count += 1;
                }
            }
            // KucoinWebsocketMsg::PongMsg(_msg) => {}
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
