use crate::event::chance::ChanceEvent;
use crate::event::order::OrderEvent;
use crate::event::trade::TradeEvent;
use crate::model::order::{LimitOrder, OrderType};
use std::time::SystemTime;
use tokio::sync::broadcast::{Receiver, Sender};
use uuid::Uuid;
// TODO implement when all_taker_btc_usdt is done
// TODO implement profit maximization

/// Broker that accepts chances, then outputs actual orders based on other limiting factors
/// Gate Keeper
/// - Amount of money left in the account
/// - transaction formatted to tradeable digits
/// - 45 orders per 3 seconds
/// - 200 active order at once
pub async fn task_gatekeep_chances(
    mut rx_chance: Receiver<ChanceEvent>,
    mut rx_trade: Receiver<TradeEvent>,
    tx_order: Sender<OrderEvent>,
) -> Result<(), failure::Error> {
    loop {
        let status = rx_chance.recv().await;
        if let Err(e) = status {
            log::error!("gatekeep chance parsing error {e:?}");
            return Ok(());
        }
        let event: ChanceEvent = status.unwrap();
        match event {
            ChanceEvent::AllTaker(chance) => {
                log::info!("All Taker Chance found!\n{chance:?}");
                // i is [0, 1, 2]
                for i in 0..3 {
                    let uuid = Uuid::new_v4();
                    let order: LimitOrder = LimitOrder {
                        id: uuid.to_string(),
                        order_type: OrderType::Limit,
                        side: chance.actions[i].action,
                        symbol: chance.actions[i].ticker.clone(),
                        amount: chance.actions[i].volume.to_string(),
                        price: chance.actions[i].price.to_string(),
                    };
                    // Logging time
                    let time_sent = SystemTime::now();
                    log::info!("time_sent: {time_sent:?}");

                    tx_order.send(OrderEvent::PlaceLimitOrder(order))?;

                    let fill_target = chance.actions[i].price.0;
                    let mut fill_cumulative = 0.0;
                    while fill_cumulative < fill_target {
                        let trade_event = rx_trade.recv().await?;
                        match trade_event {
                            TradeEvent::TradeFilled(info) => {
                                if info.order_id.eq(&uuid.as_u128()) {
                                    // TODO use actual data to deduct the amount_untraded
                                    log::info!("roughly assume all the trade were filled!");
                                    fill_cumulative = fill_target;
                                }
                            }
                            other => {
                                log::info!("igmore [{other:?}]")
                            }
                        }
                    }

                    // wait until it receives a signal from Kucoin that the order has been complete
                }
            }
            ChanceEvent::MakerTakerTaker(_actions) => {}
        }
    }
}
