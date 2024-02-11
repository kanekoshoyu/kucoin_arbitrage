use eyre::Result;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    println!("waiting for terminating signal");
    // assume it is never ending
    let ext_signal = task_signal_handle();
    let program = program();
    tokio::select! {
        _ = ext_signal => println!("end"),
        _ = program => println!("error"),
    };
    Ok(())
}

/// task to wait for any external terminating signal
async fn task_signal_handle() -> Result<()> {
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    tokio::select! {
        _ = sigterm.recv() => exit_program("SIGTERM").await,
        _ = sigint.recv() => exit_program("SIGINT").await,
    }?;
    Ok(())
}

// Define a handler function for the SIGTERM signal
async fn exit_program(signal_alias: &str) -> Result<()> {
    println!("Received [{signal_alias}] signal. Cleaning up and shutting down gracefully.");
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
