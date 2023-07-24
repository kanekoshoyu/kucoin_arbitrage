use std::sync::Arc;

use crate::event::chance::ChanceEvent;
use crate::event::order::OrderEvent;
use crate::event::orderchange::OrderChangeEvent;
use crate::global::counter_helper;
use crate::model::counter::Counter;
use crate::model::order::{LimitOrder, OrderType};
use crate::strings::generate_uid;
use std::time::SystemTime;
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
    mut receiver_chance: Receiver<ChanceEvent>,
    mut receiver_order_change: Receiver<OrderChangeEvent>,
    sender: Sender<OrderEvent>,
    counter: Arc<Mutex<Counter>>,
) -> Result<(), kucoin_api::failure::Error> {
    loop {
        counter_helper::increment(counter.clone()).await;
        let status = receiver_chance.recv().await;
        if let Err(e) = status {
            log::error!("gatekeep chance parsing error {e:?}");
            return Ok(());
        }
        let event: ChanceEvent = status.unwrap();
        match event {
            ChanceEvent::AllTaker(chance) => {
                log::info!("All Taker Chance found!\n{chance:?}");
                // TODO conduct profit maximization here
                // set up a sized queue here with a timer and a order monitor
                // if timeout, close order with market price
                // chance.profit
                // i is [0, 1, 2]
                for i in 0..3 {
                    let order: LimitOrder = LimitOrder {
                        id: generate_uid(40),
                        order_type: OrderType::Limit,
                        side: chance.actions[i].action,
                        symbol: chance.actions[i].ticker.clone(),
                        amount: chance.actions[i].volume.to_string(),
                        price: chance.actions[i].price.to_string(),
                    };
                    // Logging time
                    let time_sent = SystemTime::now();
                    log::info!("time_sent: {time_sent:?}");

                    sender.send(OrderEvent::PostOrder(order)).unwrap();

                    let mut amount_untraded = chance.actions[i].price.0;
                    while amount_untraded > 0.0 {
                        let order_change_status = receiver_order_change.recv().await;
                        if order_change_status.is_err() {
                            log::error!(
                                "gatekeep change parsing error {:?}",
                                order_change_status.err().unwrap()
                            );
                            continue;
                        }
                        let order_change_event = order_change_status.unwrap();
                        if let OrderChangeEvent::OrderFilled((amount, currency)) =
                            order_change_event
                        {
                            log::info!("{amount}{currency} filled, proceeding to next step");
                            amount_untraded = 0.0;
                        }
                    }

                    // wait until it receives a signal from Kucoin that the order has been complete
                }
            }
            ChanceEvent::MakerTakerTaker(_actions) => {}
        }
    }
}
