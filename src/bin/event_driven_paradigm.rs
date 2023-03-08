use kucoin_rs::tokio;
use kucoin_rs::tokio::sync::broadcast::{self, Receiver, Sender};
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
struct Event<T> {
    alias: String,
    data: T,
}

trait EventHandler{
    
}

impl Event<u32> {
    fn new(value: &str) -> Self {
        Event {
            alias: String::from(value),
            data: 1,
        }
    }

    fn print() {}
}

async fn run(mut receiver: Receiver<Event<u32>>) -> Result<(), Box<dyn Error>> {
    while let Ok(event) = receiver.recv().await {
        println!("Received event: {}", event.alias);
        // Do something with the event
    }
    Ok(())
}

async fn run(mut receiver: Receiver<Event<u32>>) -> Result<(), Box<dyn Error>> {
    while let Ok(event) = receiver.recv().await {
        println!("Received event: {}", event.alias);
        // Do something with the event
    }
    Ok(())
}

// Event Manager



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (sender, receiver) = broadcast::channel::<Event>(32);
    tokio::spawn(async move {
        if let Err(err) = run(receiver).await {
            eprintln!("Error: {}", err);
        }
    });
    sender.send(Event::new("Event 1"))?;
    sender.send(Event::new("Event 2"))?;
    sender.send(Event::new("Event 3"))?;
    Ok(())
}
