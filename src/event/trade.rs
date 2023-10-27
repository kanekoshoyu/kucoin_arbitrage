/// Trade status received from exchange
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TradeEvent {
    TradeOpen((u128, String)),
    TradeFilled((u128, String)),
    TradeCanceled((u128, String)),
}
