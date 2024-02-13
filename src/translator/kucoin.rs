/*
    Translates from kucoin_api crates model to out internal model
*/

use std::str::FromStr;

use crate::model;
use crate::translator::traits;
use eyre::Result;
use kucoin_api::model as api_model;
use ordered_float::OrderedFloat;
use uuid::Uuid;

impl traits::ToOrderBook for api_model::market::OrderBook {
    fn to_internal(&self) -> model::orderbook::Orderbook {
        let parse_err_msg = "Failed to parse input";
        let sequence = self.sequence.parse::<u64>().unwrap();
        let mut ask = model::orderbook::PVMap::new();
        let mut bid = model::orderbook::PVMap::new();

        for ask_pv in self.asks.clone() {
            let price: OrderedFloat<f64> = ask_pv[0].parse().expect(parse_err_msg);
            let volume: OrderedFloat<f64> = ask_pv[1].parse().expect(parse_err_msg);
            ask.insert(price, volume);
        }
        for bid_pv in self.bids.clone() {
            let price: OrderedFloat<f64> = bid_pv[0].parse().expect(parse_err_msg);
            let volume: OrderedFloat<f64> = bid_pv[1].parse().expect(parse_err_msg);
            bid.insert(price, volume);
        }
        model::orderbook::Orderbook { ask, bid, sequence }
    }
}

impl traits::ToOrderBookChange for api_model::websocket::Level2 {
    /// converts to (symbol, orderbook)
    fn to_internal(&self, last_serial: u64) -> (String, model::orderbook::Orderbook) {
        // return Orderbook::new();
        let mut ask = model::orderbook::PVMap::new();
        let mut bid = model::orderbook::PVMap::new();
        let parse_err_msg = "Failed to parse input";

        for ask_change in self.changes.asks.clone() {
            // ignore if sequence <=serial
            if ask_change[2].parse::<u64>().unwrap() > last_serial {
                let price: OrderedFloat<f64> = ask_change[0].parse().expect(parse_err_msg);
                let volume: OrderedFloat<f64> = ask_change[1].parse().expect(parse_err_msg);
                ask.insert(price, volume);
            }
        }
        for bid_change in self.changes.bids.clone() {
            // ignore if sequence <=serial
            if bid_change[2].parse::<u64>().unwrap() > last_serial {
                let price: OrderedFloat<f64> = bid_change[0].parse().expect(parse_err_msg);
                let volume: OrderedFloat<f64> = bid_change[1].parse().expect(parse_err_msg);
                bid.insert(price, volume);
            }
        }
        let sequence = self.sequence_end as u64;
        (
            self.symbol.clone(),
            model::orderbook::Orderbook { ask, bid, sequence },
        )
    }
}

impl traits::ToSymbolInfo for api_model::market::SymbolList {
    fn to_internal(&self) -> model::symbol::SymbolInfo {
        model::symbol::SymbolInfo {
            symbol: self.symbol.clone(),
            base: self.base_currency.clone(),
            quote: self.quote_currency.clone(),
            base_increment: self.base_increment.parse().unwrap(),
            base_min: self.base_min_size.parse().unwrap(),
        }
    }
}

impl traits::ToTradeInfo for api_model::websocket::TradeReceived {
    fn to_internal(&self) -> Result<model::trade::TradeInfo> {
        let order_id = Uuid::parse_str(&self.client_oid)?.as_u128();
        let symbol = self.symbol.clone();
        let side = model::order::OrderSide::from_str(self.side.as_ref())?;
        let size = self.size.clone();
        let order_type: model::order::OrderType =
            model::order::OrderType::from_str(self.order_type.as_ref())?;
        Ok(model::trade::TradeInfo {
            order_id,
            symbol,
            side,
            order_type,
            size,
        })
    }
}

impl traits::ToTradeInfo for api_model::websocket::TradeOpen {
    fn to_internal(&self) -> Result<model::trade::TradeInfo> {
        let order_id = Uuid::parse_str(&self.client_oid)?.as_u128();
        let symbol = self.symbol.clone();
        let side = model::order::OrderSide::from_str(self.side.as_ref())?;
        let size = self.size.clone();
        let order_type: model::order::OrderType =
            model::order::OrderType::from_str(self.order_type.as_ref())?;
        Ok(model::trade::TradeInfo {
            order_id,
            symbol,
            side,
            order_type,
            size,
        })
    }
}

impl traits::ToTradeInfo for api_model::websocket::TradeFilled {
    fn to_internal(&self) -> Result<model::trade::TradeInfo> {
        let order_id = Uuid::parse_str(&self.client_oid)?.as_u128();
        let symbol = self.symbol.clone();
        let side = model::order::OrderSide::from_str(self.side.as_ref())?;
        let size = self.size.clone();
        let order_type: model::order::OrderType =
            model::order::OrderType::from_str(self.order_type.as_ref())?;
        Ok(model::trade::TradeInfo {
            order_id,
            symbol,
            side,
            order_type,
            size,
        })
    }
}

impl traits::ToTradeInfo for api_model::websocket::TradeMatch {
    fn to_internal(&self) -> Result<model::trade::TradeInfo> {
        let order_id = Uuid::parse_str(&self.client_oid)?.as_u128();
        let symbol = self.symbol.clone();
        let side = model::order::OrderSide::from_str(self.side.as_ref())?;
        let size = self.size.clone();
        let order_type: model::order::OrderType =
            model::order::OrderType::from_str(self.order_type.as_ref())?;
        Ok(model::trade::TradeInfo {
            order_id,
            symbol,
            side,
            order_type,
            size,
        })
    }
}

impl traits::ToTradeInfo for api_model::websocket::TradeCanceled {
    fn to_internal(&self) -> Result<model::trade::TradeInfo> {
        let order_id = Uuid::parse_str(&self.client_oid)?.as_u128();
        let symbol = self.symbol.clone();
        let side = model::order::OrderSide::from_str(self.side.as_ref())?;
        let size = self.size.clone();
        let order_type: model::order::OrderType =
            model::order::OrderType::from_str(self.order_type.as_ref())?;
        Ok(model::trade::TradeInfo {
            order_id,
            symbol,
            side,
            order_type,
            size,
        })
    }
}
