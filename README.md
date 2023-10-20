# Event-Driven KuCoin Triangular Arbitrage Framework in Async Rust
[![](https://img.shields.io/crates/v/kucoin-arbitrage)](https://crates.io/crates/kucoin-arbitrage)
[![](https://img.shields.io/docsrs/kucoin_arbitrage)](https://docs.rs/kucoin_arbitrage)
[![](https://img.shields.io/github/license/kanekoshoyu/kucoin_arbitrage)](https://github.com/kanekoshoyu/kucoin_arbitrage/blob/master/LICENSE)  
This is an async Rust project to implement zero-risk crypto trinagular arbitrage, explore technical feasiblity of generating passsive income (i.e. sleep to earn!).  
## How the arbitrage works
Say we hold USDT, it checks all the coins(e.g. ETH) that can trade against BTC and USDT, and compare the profit by either:  
- Buy-Sell-Sell: buy ETH (sell USDT), sell ETH (buy BTC), sell BTC (buy USDT)  
- Buy-Buy-Sell: buy BTC (sell USDT), buy ETH (sell BTC), sell ETH (buy USDT)  
  
## Previous mistakes in Python
2 years ago I have made a python script that runs the triangular arbitrage in KuCoin, but it had several technical issues and ended up not following up.  
https://github.com/kanekoshoyu/Kucoin-Triangular-Arbitrage  
- Implementation with Python and REST polling was way too slow to obtain the valid arbitratge chances for execution.
- Generally the Python REST API caused quite high rates of communication error, which took extra time for resorting.
- It didn't count the actual size of the arbitrage order, which meant that the script kept buying some shitcoin and could not sell properly.
- It simply took the mean price instead of best bid/ask, which means it took maker positions for each of three actions in one arbitrage, and did not execute arbitrage promptly.
## How to run example executables
Copy/Rename config_sample.toml as config.toml and set the API key with your own KuCoin API credentials. Configure the event monitor interval and the USD budget per clyclic arbitrage.   
```
[KuCoin Credentials]
api_key="YOUR_API_KEY"
secret_key="YOUR_SECRET_KEY"
passphrase="YOUR_PASSPHRASE"

[Behaviour]
# Performance monitor interval in seconds
monitor_interval_sec=120
# max amount of USD to use in a single cyclic arbitrage
usd_cyclic_arbitrage=100
```
At the root directory of the project, run the below command
```
cargo run --bin event_triangular  
```
`event_triangular` is one of the example executables. There are other executables in the `bin` directory.

## Overview
The project is split into these components:
###### Example Executables
- `bin` contains example executable codes. Some of them are for network testing purpose.
###### Internal Structure (Independent of Exchange APIs)
- `model` has internal generic data structures used for abstracted representations of markets. This should be independent of exchange APIs so that the the arbitrage strategy algorithm can be conducted across different exchanges.
- `event` has the events used to pass states and data passed across different components. It uses the internal model for the same reason.
- `strategy` has the implementations of arbitrage strategy algorithm. The algorithms are built upon internal model and event. 
- `monitor` has the counter used to monitor MPS (message per seconds) of each broadcast channels, and a timers mapped globally by string for easy debug access. 
  
###### Link to Exchange APIs (e.g. KuCoin)
- `translator` has the conversion of exchange API objects into internal models and vice versa. It uses traits and the traits are implemented per API models.
- `broker` has the tasks that runs API calls, and converts into internal data structure.

## Major Structural Improvements
- Use compiled Rust code for neat, efficient and proper code
- WebSocket subscription of real-time order books to get all the latest maker/taker ask/bid
- Only take a taker position based on the latest best-bid/ask price/size
- Implement both data bandwidth monitor and arbitrage performance monitor as tasks
- Abstraction of orderbook sync and arbitrage strategies using internal model and event
- Concurrency of syncing and strategy tasks
  
## Feature Progress List
| Feature                                                                                            | Status    |
| -------------------------------------------------------------------------------------------------- | --------- |
| Whitelist all coins that can trade against two other quote coins (e.g. ETH, for ETH-BTC, ETH-USDT) | Available |
| Look for arbitrage chance based on best ask/bid price and calculate the profit in percentage       | Available |
| Copy and sync local orderbook in real-time                                                         | Available |
| Structurally allow multiple strategies to run in pararrel                                          | Available |
| Order placement upon triangular arbitrage chance                                                   | Available |
| Resort against limit order that could not get filled                                               | Pending   |
| Full triangular arbitrage with the middle coin other than BTC (e.g. ETH-USD, ALT-ETH, ALT-USD)     | Pending   |

  
## To-dos
please refer to the [Discussions](https://github.com/kanekoshoyu/kucoin_arbitrage/discussions) channel in GitHub.