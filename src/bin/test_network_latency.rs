use tokio::net::TcpStream;
use tokio::time::{Duration, Instant};

async fn ping_tcp(host: &str, port: u16, timeout: Duration) -> Result<Duration, std::io::Error> {
    let address = format!("{}:{}", host, port);
    let start_time = Instant::now();

    match tokio::time::timeout(timeout, TcpStream::connect(&address)).await {
        Ok(Ok(_)) => {
            let rtt = Instant::now().duration_since(start_time);
            Ok(rtt)
        }
        Ok(Err(err)) => Err(err),
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Connection attempt timed out",
        )),
    }
}

#[tokio::main]
async fn main() {
    let target_host = "example.com";
    let target_port = 80;
    let timeout_duration = Duration::from_secs(5); // Set your desired timeout in seconds.

    match ping_tcp(target_host, target_port, timeout_duration).await {
        Ok(rtt) => println!("Ping successful. RTT: {:?}", rtt),
        Err(err) => println!("Ping failed. Error: {:?}", err),
    }
}
