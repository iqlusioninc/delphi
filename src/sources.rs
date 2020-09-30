//! Data sources

pub mod alphavantage;
pub mod binance;
pub mod coinone;
pub mod gdac;
pub mod gopax;

use crate::{Error, Price, PriceQuantity};
use rust_decimal::Decimal;

/// User-Agent to send in HTTP request
pub const USER_AGENT: &str = "iqlusion delphi";

///This trait allows writing generic functions over ask orderbook from multiple sources
pub trait AskBook {
    ///This function returns a vector of ask prices and volumes
    fn asks(&self) -> Result<Vec<PriceQuantity>, Error>;
}

///This trait allows writing generic functions over bid orderbook from multiple sources
pub trait BidBook {
    ///This function returns a vector of bid prices and volumes
    fn bids(&self) -> Result<Vec<PriceQuantity>, Error>;
}

/// Ask price weighted average
pub fn weighted_avg_ask<T: AskBook>(asks: &T) -> Result<Price, Error> {
    let asks = asks.asks()?;
    let mut price_sum_product = Decimal::from(0u8);
    let mut total = Decimal::from(0u8);
    for ask in asks {
        price_sum_product += *ask.price * ask.quantity;
        total += ask.quantity;
    }

    let weighted_avg = Price::new(price_sum_product / total)?;
    Ok(weighted_avg)
}

/// Bid price weighted average
pub fn weighted_avg_bid<T: BidBook>(bids: &T) -> Result<Price, Error> {
    let bids = bids.bids()?;
    let mut price_sum_product = Decimal::from(0u8);
    let mut total = Decimal::from(0u8);
    for bid in bids {
        price_sum_product += *bid.price * bid.quantity;
        total += bid.quantity;
    }

    let weighted_avg = Price::new(price_sum_product / total)?;
    Ok(weighted_avg)
}

/// Highest ask price
pub fn lowest_ask<T: AskBook>(asks: &T) -> Result<Price, Error> {
    let mut asks = asks.asks()?;
    asks.sort();
    Ok(asks.first().unwrap().price)
}

/// Lowest bid price
pub fn highest_bid<T: BidBook>(bids: &T) -> Result<Price, Error> {
    let mut bids = bids.bids()?;
    bids.sort();
    Ok(bids.last().unwrap().price)
}

/// Midpoint of highest ask and lowest bid price
pub fn midpoint<T: AskBook + BidBook>(book: &T) -> Result<Price, Error> {
    let lowest_ask = lowest_ask(book)?;
    let highest_bid = highest_bid(book)?;
    Ok((lowest_ask + highest_bid) / 2)
}
