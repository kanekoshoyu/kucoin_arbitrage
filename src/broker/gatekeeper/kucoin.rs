use crate::event::chance::ChanceEvent;
use crate::event::order::OrderEvent;
use crate::event::trade::TradeEvent;
use crate::model::order::{LimitOrder, OrderType};
use eyre::Result;
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
) -> Result<()> {
    loop {
        let status = rx_chance.recv().await;
        if let Err(e) = status {
            log::error!("gatekeep chance parsing error {e:?}");
            return Ok(());
        }
        let event: ChanceEvent = status.unwrap();
        // TODO timeout mechanism
        match event {
            ChanceEvent::AllTaker(chance) => {
                log::info!("All taker chance found!");
                log::info!("profit: {}", chance.profit);
                for action in &chance.actions {
                    log::info!("{action:?}");
                }
                // i is [0, 1, 2]
                for i in 0..3 {
                    let uuid = Uuid::new_v4();
                    // TODO check if the is any problem with the DP format with API
                    let order: LimitOrder = LimitOrder {
                        id: uuid.to_string(),
                        order_type: OrderType::Limit,
                        side: chance.actions[i].action,
                        symbol: chance.actions[i].ticker.clone(),
                        amount: format!("{:.9}", chance.actions[i].volume),
                        price: format!("{:.9}", chance.actions[i].price),
                    };
                    tx_order.send(OrderEvent::PlaceLimitOrder(order))?;
                    let fill_target = chance.actions[i].price.0;
                    let mut fill_cumulative = 0.0;
                    while fill_cumulative < fill_target {
                        log::info!("Waiting for TradeInfo from KuCoin server");
                        let trade_event = rx_trade.recv().await?;
                        match trade_event {
                            TradeEvent::TradeFilled(info) => {
                                if info.order_id.eq(&uuid.as_u128()) {
                                    // TODO use actual data to deduct the amount_untraded
                                    let fill_size: f64 = info.size.parse()?;
                                    fill_cumulative += fill_size;
                                    log::info!(
                                        "Filled [{fill_cumulative}/{fill_target}] of {:?}",
                                        info.symbol
                                    );
                                }
                            }
                            TradeEvent::TradeCanceled(info) => {
                                if info.order_id.eq(&uuid.as_u128()) {
                                    log::warn!("Trade got canceled [{}]", info.order_id);
                                    break;
                                }
                            }
                            other => {
                                // print for debugging purpose
                                if let TradeEvent::TradeMatch(info) = other {
                                    log::info!("Ignoring TradeMatch[{}]", info.order_id);
                                } else {
                                    log::info!("Ignoring [{other:?}]");
                                }
                            }
                        }
                    }
                }
                log::info!("cycle completed!")
            }
            ChanceEvent::MakerTakerTaker(_actions) => {}
        }
    }
}
