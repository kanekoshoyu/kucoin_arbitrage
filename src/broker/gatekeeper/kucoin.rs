use std::sync::Arc;

use crate::event::order::OrderEvent;
use crate::global::counter_helper;
use crate::model::counter::Counter;
use crate::model::order::LimitOrder;
use crate::{event::chance::ChanceEvent, model::order::OrderType};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

// TODO implement when all_taker_btc_usdt is done

/// Broker that accepts chances, then outputs actual orders based on other limiting factors
/// Gate Keeper
/// - Amount of money left in the account
/// - transaction formatted to tradeable digits
/// - 45 orders per 3 seconds
/// - 200 active order at once
pub async fn task_gatekeep_chances(
    mut receiver: Receiver<ChanceEvent>,
    sender: Sender<OrderEvent>,
    counter: Arc<Mutex<Counter>>,
) -> Result<(), kucoin_api::failure::Error> {
    let mut serial: u128 = 0;
    loop {
        counter_helper::increment(counter.clone()).await;
        let status = receiver.recv().await;
        if status.is_err() {
            log::error!("task_gatekeep_chances error {:?}", status.err().unwrap());
            continue;
        }
        let event: ChanceEvent = status.unwrap();
        match event {
            ChanceEvent::AllTaker(chance) => {
                log::info!("All Taker Chance found!\n{chance:?}");
                // TODO conduct profit maximization here
                // set up a sized queue here with a timer and a order monitor
                // if timeout, close order with market price
                // TODO push to order manager

                for i in 0..2 {
                    let order: LimitOrder = LimitOrder {
                        id: serial,
                        order_type: OrderType::Limit,
                        side: chance.actions[i].action,
                        symbol: chance.actions[i].ticker.clone(),
                        amount: chance.actions[i].volume.to_string(),
                        price: chance.actions[i].price.to_string(),
                    };
                    serial += 1;
                    sender.send(OrderEvent::PostOrder(order)).unwrap();
                }
            }
            ChanceEvent::MakerTakerTaker(_actions) => {}
        }
    }
}
