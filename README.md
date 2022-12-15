# KuCoin Triangular Arbitrage Bot in Rust
Rust project to implement zero-risk crypto trinagular arbitrage, explore technical feasiblity of generating passsive income (i.e. sleep to earn!).  
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
## How to run
Copy config.ini.sample as config.ini and replace the API key with your own KuCoin API credentials.  
```
cargo run --bin XXX  
```
**/bin** and **main.rs** are the executable source code.  
**/globals** has the lazy_statics that is used across the code. Technically it is better to pass Arc and Mutex around the functions instead of using global statics, but I just wanted the code to be a bit more readable with this way.  
  
## Major Structural Improvements
- Use compiled Rust code for neat, efficient and  proper code
- WebSocket subscription of real-time order books to get all the latest maker/taker ask/bid
- Only take a taker position based on the latest best-bid/ask price/size
- Implement both data bandwidth monitor and arbitrage performance monitor as tasks
- Modularize the subscription sync and arbitrage strategy selections, to make it more generic
  
## Feature Progress List
| Feature                                                                                            | Status    |
| -------------------------------------------------------------------------------------------------- | --------- |
| Whitelist all coins that can trade against two other quote coins (e.g. ETH, for ETH-BTC, ETH-USDT) | Available |
| Look for arbitrage chance based on best ask/bid price and calculate the profit in percentage       | Available |
| Copy and sync local orderbook in real-time                                                         | Pending   |
| Execute on the arbitrage                                                                           | Pending   |
| Resort against trade execution that could not complete as anticipated                              | Pending   |
| Structurally allow multiple strategies to run in pararrel                                          | Pending   |
  
## To-dos
- Allow variants of triangular arbitrage to run concurrently. (e.g. all taker v.s. Maker-Taker-Taker with profit monitor, selection of multiple quote coins, spot vs futures)
- Wrap the runtime as a local service which can talk with other components in the network
- Add a Qt/WebAsssembly GUI which visualises the arbitrage situation