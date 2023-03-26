use crate::event::chance::ChanceEvent;
use crate::event::order::OrderEvent;
use crate::model::chance::{ActionInfo, ThreeActions};
use kucoin_rs::tokio::sync::broadcast::{Receiver, Sender};

/// Broker that accepts chances, then outputs actual orders based on other limiting factors
/// Gate Keeper
/// - Amount of money left in the account
/// - transaction formatted to tradeable digits
/// - 45 orders per 3 seconds
/// - 200 active order at once
pub async fn task_gatekeep_chances(
    receiver: &mut Receiver<ChanceEvent>,
    sender: &mut Sender<OrderEvent>,
) -> Result<(), kucoin_rs::failure::Error> {
    while let event = receiver.recv().await? {
        match event {
            ChanceEvent::AllTaker(actions) => {}
            ChanceEvent::MakerTakerTaker(actions) => {}
        }
    }
    // TODO: set a new Err that shows this should not have arrived
    Ok(())
}
