use eyre::Result;
use kucoin_arbitrage::system_event::task_signal_handle;
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    println!("waiting for terminating signal");
    tokio::select! {
        _ = task_signal_handle() => println!("end"),
        _ = program() => println!("error"),
    };
    Ok(())
}

// Define a handler function for the SIGTERM signal
async fn program() -> Result<()> {
    let mut counter = 0;
    let duration = time::Duration::from_secs(2);
    loop {
        println!("couter: {counter}");
        counter += 1;
        time::sleep(duration).await;
    }
}
