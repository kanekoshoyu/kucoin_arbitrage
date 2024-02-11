use std::io;
use tokio::signal::unix::{signal, SignalKind};

/// task to wait for any external terminating signal
pub async fn task_signal_handle() -> io::Result<()> {
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;
    tokio::select! {
        _ = sigterm.recv() => exit_program("SIGTERM"),
        _ = sigint.recv() => exit_program("SIGINT"),
    };
    Ok(())
}

// Define a handler function for the SIGTERM signal
fn exit_program(signal_alias: &str) {
    println!("Received [{signal_alias}] signal. Cleaning up and shutting down gracefully.");
}
