# Event-Driven KuCoin Triangular Arbitrage Framework in Async Rust
[![](https://img.shields.io/crates/v/kucoin-arbitrage)](https://crates.io/crates/kucoin-arbitrage)
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
Copy/Rename config.ini.sample as config.ini and set the API key with your own KuCoin API credentials.  
At the root directory of the project, run the below command
```
cargo run --bin event_triangular  
```
`event_triangular` is one of the example executables. There are other executables in the `bin` directory.

## Overview
The project is split into these components:
- `bin` contains example executable codes.
###### Internal Structure (Independent of Exchange APIs)
- `model` has internal generic data structures used for abstracted representations of markets. This should be generic from exchanges so that the the arbitrage algorithms can be conducted across different exchange APIs.
- `events` has the events used to pass states and data passed across different components. This should again be generic from the exchange APIs. It uses the internal model.
- `strategy` has the implementations of arbitrage algorithms. The algorithms should only be using internal model and events so that the strategy algorithms are cross platform. 
- `global` has the lazy_static globals that is used across the code. This may get replaced by passing Arc/Mutex to functions soon. I just wanted the code to be simple and readable with this way first.
  
###### Link to Exchange APIs (e.g. KuCoin)
- `translator` has the conversion of exchange API objects into internal models and vice versa. It uses traits and the traits are implemented per API models.
- `broker` has the tasks that runs API calls, and converts into internal data structure.

## Major Structural Improvements
- Use compiled Rust code for neat, efficient and proper code
- WebSocket subscription of real-time order books to get all the latest maker/taker ask/bid
- Only take a taker position based on the latest best-bid/ask price/size
- Implement both data bandwidth monitor and arbitrage performance monitor as tasks
- Abstraction and modularization of orderbook sync and arbitrage strategies using events and channel, for interoperability
  
## Feature Progress List
| Feature                                                                                            | Status    |
| -------------------------------------------------------------------------------------------------- | --------- |
| Whitelist all coins that can trade against two other quote coins (e.g. ETH, for ETH-BTC, ETH-USDT) | Available |
| Look for arbitrage chance based on best ask/bid price and calculate the profit in percentage       | Available |
| Copy and sync local orderbook in real-time                                                         | Available |
| Structurally allow multiple strategies to run in pararrel                                          | Available |
| Execute on the arbitrage                                                                           | Pending   |
| Resort against trade execution that could not complete as anticipated                              | Pending   |

  
## To-dos
please refer to the [Discussions](https://github.com/kanekoshoyu/kucoin_arbitrage/discussions) channel in GitHub.