use tokio::net::TcpStream;
use tokio::time::{Duration, Instant};

async fn ping_tcp(host: &str, port: u16, timeout: Duration) -> Result<Duration, std::io::Error> {
    let address = format!("{}:{}", host, port);
    let start_time = Instant::now();
    tokio::time::timeout(timeout, TcpStream::connect(&address)).await??;
    let rtt = Instant::now().duration_since(start_time);
    Ok(rtt)
}

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    let target_host = "api.kucoin.com";
    let target_port = 80;
    let timeout_duration = Duration::from_secs(5); // Set your desired timeout in seconds.
    loop {
        let res = ping_tcp(target_host, target_port, timeout_duration).await?;
        println!("{res:?}");
    }
}
