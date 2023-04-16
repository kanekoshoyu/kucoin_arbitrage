use crate::event::chance::ChanceEvent;
use crate::event::order::OrderEvent;
use tokio::sync::broadcast::{Receiver, Sender};

// TODO implement when all_taker_btc_usdt is done

/// Broker that accepts chances, then outputs actual orders based on other limiting factors
/// Gate Keeper
/// - Amount of money left in the account
/// - transaction formatted to tradeable digits
/// - 45 orders per 3 seconds
/// - 200 active order at once
pub async fn task_gatekeep_chances(
    mut receiver: Receiver<ChanceEvent>,
    mut _sender: Sender<OrderEvent>,
) -> Result<(), kucoin_rs::failure::Error> {
    loop {
        let status = receiver.recv().await;
        if status.is_err() {
            // TODO return the error as we want here
            panic!("{:?}", status.err());
        }
        let event = status.unwrap();
        match event {
            ChanceEvent::AllTaker(_actions) => {}
            ChanceEvent::MakerTakerTaker(_actions) => {}
        }
    }
}
