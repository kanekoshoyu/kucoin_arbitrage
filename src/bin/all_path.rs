use eyre::Result;
use interning::InternedString;
use kucoin_api::model::market::SymbolList;
use kucoin_arbitrage::system_event::task_signal_handle;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
// TODO suggest a data structure which we can use to store all the paths and search those paths efficiently
// TODO use interned string to store the asset symbol as u64
// TODO directly write a function from the actual data to verify them.

#[tokio::main]
async fn main() -> Result<()> {
    println!("exit upon terminating signal");
    tokio::select! {
        _ = task_signal_handle() => eyre::bail!("end"),
        _ = program() => Ok(()),
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
        write!(f, "{:?}({:?})", self.action, self.pair)
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
}
impl Debug for TradeCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cycle{:?}", self.actions)
    }
}

// all cycles can be used to look up for trade cycles with the asset ID
pub type AllCycles = HashMap<u64, Vec<TradeCycle>>;

////////////////////////////// fn

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
    pub fn dfs(&mut self, current: u64, graph: &HashMap<u64, Vec<Pair>>, start_with_buy: bool) {
        self.visited.insert(current);
        if let Some(pairs) = graph.get(&current) {
            for pair in pairs {
                let (next_node, action) = if current == pair.base {
                    (pair.quote, Action::Sell)
                } else {
                    (pair.base, Action::Buy)
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
                            pair: pair.clone(),
                            action,
                        });
                        self.found_cycles.push(cycle);
                    } else if !self.visited.contains(&next_node) {
                        self.cycle.push(TradeAction {
                            pair: pair.clone(),
                            action,
                        });
                        self.dfs(next_node, graph, false); // After the first trade, no need to enforce Buy as start.
                        self.cycle.pop();
                    }
                }
            }
        }
        self.visited.remove(&current);
    }
    /// generate all the cyclic paths from the Graph
    fn find_cycles(
        &mut self,
        pairs: impl IntoIterator<Item = Pair>,
        start: u64,
    ) -> Vec<TradeCycle> {
        // Constructing the graph from Pair structs
        let mut graph: HashMap<u64, Vec<Pair>> = HashMap::new();
        for pair in pairs {
            graph
                .entry(pair.base)
                .or_insert_with(Vec::new)
                .push(pair.clone());
            graph.entry(pair.quote).or_insert_with(Vec::new).push(pair);
        }
        self.start = start;
        // Start DFS with the flag to ensure the first trade is a Buy.
        self.dfs(start, &graph, true);
        self.found_cycles.clone()
    }
}

async fn program() -> Result<()> {
    // kucoin api endpoints
    let api = Kucoin::new(KucoinEnv::Live, Some(config.kucoin_credentials()))
        .map_err(|e| eyre::eyre!(e))?;
    tracing::info!("Credentials setup");
    let symbol_list = api.get_symbol_list(None).await;
    let symbol_list = symbol_list.data.expect("empty symbol list");
    let pairs = symbol_list.iter().map(Pair::from).collect();

    let start_node = 1u64;
    let mut finder = CycleFinder::new();
    let found_cycles = finder.find_cycles(pairs, start_node);
    for (index, path) in found_cycles.iter().enumerate() {
        println!("Path {}: {:?}", index + 1, path);
    }
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
