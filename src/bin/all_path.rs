use eyre::Result;
use interning::{InternedString, InternedStringHash};
use kucoin_api::client::{Kucoin, KucoinEnv};
use kucoin_api::model::market::SymbolList;
use kucoin_arbitrage::system_event::task_signal_handle;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

#[tokio::main]
async fn main() -> Result<()> {
    println!("exit upon terminating signal");
    let config = kucoin_arbitrage::config::from_file("config.toml")?;
    tokio::select! {
        _ = task_signal_handle() => eyre::bail!("end"),
        _ = core(config) => Ok(()),
    }
}

////////////////////////////// struct

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct Pair {
    base: u64,
    quote: u64,
}
impl Debug for Pair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.base, self.quote)
    }
}
impl std::fmt::Display for Pair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base = hash_to_string(self.base);
        let quote = hash_to_string(self.quote);
        write!(f, "{}-{}", base, quote)
    }
}
impl From<SymbolList> for Pair {
    fn from(value: SymbolList) -> Self {
        let base = InternedString::from(value.base_currency);
        let base = base.hash().hash();
        let quote = InternedString::from(value.quote_currency);
        let quote = quote.hash().hash();
        Pair { base, quote }
    }
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Action {
    Buy,
    Sell,
}

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct TradeAction {
    pair: Pair,
    action: Action,
}
impl Debug for TradeAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({})", self.action, self.pair)
    }
}
impl TradeAction {
    pub fn buy(base: u64, quote: u64) -> Self {
        TradeAction {
            pair: Pair { base, quote },
            action: Action::Buy,
        }
    }
    pub fn sell(base: u64, quote: u64) -> Self {
        TradeAction {
            pair: Pair { base, quote },
            action: Action::Sell,
        }
    }
}
#[derive(Clone, Hash, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct TradeCycle {
    actions: Vec<TradeAction>,
}
impl From<Vec<TradeAction>> for TradeCycle {
    fn from(actions: Vec<TradeAction>) -> Self {
        TradeCycle { actions }
    }
}
impl TradeCycle {
    pub fn new() -> Self {
        TradeCycle::default()
    }
    pub fn push(&mut self, trade_action: TradeAction) {
        self.actions.push(trade_action)
    }
    pub fn pop(&mut self) -> Option<TradeAction> {
        self.actions.pop()
    }
    pub fn len(&self) -> usize {
        self.actions.len()
    }
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}
impl Debug for TradeCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cycle{:?}", self.actions)
    }
}

// all cycles can be used to look up for trade cycles with the asset ID
pub type AllCycles = HashMap<u64, Vec<TradeCycle>>;

////////////////////////////// fn

pub fn hash_to_string(id: u64) -> String {
    let to_hash = |id: u64| InternedStringHash::from_bytes(id.to_be_bytes());
    unsafe { InternedString::from_hash(to_hash(id)) }.to_string()
}

// TODO this searches AVA only
// pairs in, cycles out
#[derive(Clone, Default)]
struct CycleFinder {
    start: u64,
    cycle: TradeCycle,
    visited: HashSet<u64>,
    found_cycles: Vec<TradeCycle>,
}
impl CycleFinder {
    pub fn new() -> Self {
        CycleFinder::default()
    }
    /// search function
    fn dfs(&mut self, current: u64, graph: &HashMap<u64, Vec<Pair>>, start_with_buy: bool) {
        if !self.visited.insert(current) {
            // skip if current id was contained before
            return;
        }
        let pairs = graph.get(&current).expect("no pair found");
        
        tracing::info!("dfs({})", hash_to_string(current));
        for pair in pairs {
            let next_node = if current == pair.base {
                pair.quote
            } else {
                pair.base
            };
            let action = if current == pair.base {
                Action::Sell
            } else {
                Action::Buy
            };

            // Enforce "Buy before Sell" rule: If path is empty, start only with Buy. Otherwise, proceed as per the action.
            if !start_with_buy || action == Action::Buy {
                if next_node == self.start
                    && self
                        .cycle
                        .actions
                        .iter()
                        .any(|trade| trade.action == Action::Buy)
                {
                    let mut cycle = self.cycle.clone();
                    cycle.push(TradeAction {
                        pair: *pair,
                        action,
                    });
                    self.found_cycles.push(cycle);
                } else if !self.visited.contains(&next_node) {
                    self.cycle.push(TradeAction {
                        pair: *pair,
                        action,
                    });
                    self.dfs(next_node, graph, false); // After the first trade, no need to enforce Buy as start.
                    self.cycle.pop();
                }
            }
        }
    }
    /// generate all the cyclic paths from the Graph
    pub fn find_cycles(
        &mut self,
        pairs: impl IntoIterator<Item = Pair>,
        start: u64,
    ) -> Vec<TradeCycle> {
        // populate graph from pairs
        let mut graph: HashMap<u64, Vec<Pair>> = HashMap::new();
        for pair in pairs {
            graph.entry(pair.base).or_default().push(pair);
            graph.entry(pair.quote).or_default().push(pair);
        }
        self.start = start;
        self.visited.clear();
        self.found_cycles.clear();
        // Start DFS with the flag to ensure the first trade is a Buy
        self.dfs(start, &graph, true);
        std::mem::take(&mut self.found_cycles)
    }
}

async fn core(config: kucoin_arbitrage::config::Config) -> Result<()> {
    let _worker_guard = kucoin_arbitrage::logger::setup_logs(&config.log)?;
    // kucoin api endpoints
    let api = Kucoin::new(KucoinEnv::Live, Some(config.kucoin_credentials()))
        .map_err(|e| eyre::eyre!(e))?;

    tracing::info!("credentials setup");
    let symbol_list = api.get_symbol_list(None).await;
    let symbol_list = symbol_list.expect("failed receiving data from exchange");
    let symbol_list = symbol_list.data.expect("empty symbol list");
    let pairs: Vec<Pair> = symbol_list.into_iter().map(Pair::from).collect();
    tracing::info!("{}pairs found", pairs.len());
    // usd as starting node
    let start_node = InternedString::from_str("USDT");
    let start_node = start_node.hash().hash();
    let mut finder = CycleFinder::new();
    let found_cycles: Vec<TradeCycle> = finder.find_cycles(pairs, start_node);
    let cycle_count = |x: &TradeCycle| x.len() == 3;
    let found_cycles: Vec<TradeCycle> = found_cycles.into_iter().filter(cycle_count).collect();
    if found_cycles.is_empty() {
        tracing::info!("no cycles found");
    } else {
        for (index, path) in found_cycles.iter().enumerate() {
            tracing::info!("Path {}: {:?}", index + 1, path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfs() {
        // Setup a simple graph that represents the trading pairs.
        let pairs = vec![
            Pair { base: 1, quote: 2 },
            Pair { base: 2, quote: 3 },
            Pair { base: 3, quote: 1 },
            Pair { base: 2, quote: 4 },
            Pair { base: 4, quote: 1 },
        ];

        let start_node = 1u64;
        let mut finder = CycleFinder::new();
        let actual_cycles = finder.find_cycles(pairs, start_node);
        // Define the expected paths using the Trade struct.
        // Note: The expected paths should match the actual trading paths you expect based on your graph setup.
        let expected_cycles = vec![
            TradeCycle::from(vec![
                TradeAction::buy(3, 1),
                TradeAction::buy(2, 3),
                TradeAction::buy(1, 2),
            ]),
            TradeCycle::from(vec![
                TradeAction::buy(3, 1),
                TradeAction::buy(2, 3),
                TradeAction::sell(2, 4),
                TradeAction::sell(4, 1),
            ]),
            TradeCycle::from(vec![TradeAction::buy(3, 1), TradeAction::sell(3, 1)]),
            TradeCycle::from(vec![
                TradeAction::buy(4, 1),
                TradeAction::buy(2, 4),
                TradeAction::buy(1, 2),
            ]),
            TradeCycle::from(vec![TradeAction::buy(4, 1), TradeAction::sell(4, 1)]),
            TradeCycle::from(vec![
                TradeAction::buy(4, 1),
                TradeAction::buy(2, 4),
                TradeAction::sell(2, 3),
                TradeAction::sell(3, 1),
            ]),
        ];
        // TODO might better off writing a custom cmp function with Vec<TradeCycle>
        let actual: HashSet<TradeCycle> = actual_cycles.into_iter().collect();
        let expected: HashSet<TradeCycle> = expected_cycles.into_iter().collect();

        // Check if the trading paths found match the expected paths.
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_dfs_case_2() {
        // Setup a simple graph that represents the trading pairs.
        let pairs = vec![
            Pair { base: 1, quote: 2 },
            Pair { base: 2, quote: 3 },
            Pair { base: 3, quote: 1 },
        ];

        let start_node = 1u64;
        let mut finder = CycleFinder::new();
        let actual_cycles = finder.find_cycles(pairs, start_node);

        // Define the expected paths using the Trade struct.
        // Note: The expected paths should match the actual trading paths you expect based on your graph setup.
        let expected_cycles = vec![
            TradeCycle::from(vec![
                TradeAction::buy(3, 1),
                TradeAction::buy(2, 3),
                TradeAction::buy(1, 2),
            ]),
            TradeCycle::from(vec![TradeAction::buy(3, 1), TradeAction::sell(3, 1)]),
        ];

        // Check if the trading paths found match the expected paths.
        assert_eq!(actual_cycles, expected_cycles);
    }
}
