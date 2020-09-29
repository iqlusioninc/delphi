//! Data sources

pub mod binance;
pub mod coinone;
pub mod gdac;
pub mod gopax;

use crate::{
    error::{Error, ErrorKind},
    prelude::*,
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use rust_decimal::{prelude::*, Decimal};
use serde::{
    de::{self, Error as _},
    ser, Deserialize, Serialize,
};
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    str::FromStr,
};

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
        price_sum_product += ask.price.0 * ask.quantity;
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
        price_sum_product += bid.price.0 * bid.quantity;
        total += bid.quantity;
    }

    let weighted_avg = Price::new(price_sum_product / total)?;
    Ok(weighted_avg)
}

/// Highest ask price
pub fn lowest_ask<T: AskBook>(asks: &T) -> Result<Price, Error> {
    let mut asks = asks.asks()?;
    asks.sort();
    Ok(asks.first().unwrap().price.clone())
}

/// Lowest bid price
pub fn highest_bid<T: BidBook>(bids: &T) -> Result<Price, Error> {
    let mut bids = bids.bids()?;
    bids.sort();
    Ok(bids.last().unwrap().price.clone())
}

/// Midpoint of highest ask and lowest bid price
pub fn midpoint<T: AskBook + BidBook>(book: &T) -> Result<Price, Error> {
    let lowest_ask = lowest_ask(book)?;
    let highest_bid = highest_bid(book)?;
    Price::new((lowest_ask.0 + highest_bid.0) / Decimal::from(2))
}

/// Quoted prices and quantities as sourced from the order book
#[derive(Eq)]
pub struct PriceQuantity {
    ///Price
    pub price: Price,

    ///Quantity
    pub quantity: Decimal,
}

impl Ord for PriceQuantity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price.cmp(&other.price)
    }
}

impl PartialOrd for PriceQuantity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PriceQuantity {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price
    }
}

/// Currencies for use in trading pairs
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Currency {
    /// South Korean won
    Krw,

    /// Terra Luna
    Luna,

    /// Other (open-ended)
    Other(String),
}

impl Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Currency::Krw => "KRW",
            Currency::Luna => "LUNA",
            Currency::Other(other) => other.as_ref(),
        })
    }
}

impl FromStr for Currency {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(match s.to_ascii_uppercase().as_ref() {
            "KRW" => Currency::Krw,
            "LUNA" => Currency::Luna,
            other => Currency::Other(other.to_owned()),
        })
    }
}

impl<'de> Deserialize<'de> for Currency {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl Serialize for Currency {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

/// Trading pairs
pub struct Pair(pub Currency, pub Currency);

impl Pair {
    /// Percent encode this pair (for inclusion in a URL)
    pub fn percent_encode(&self) -> String {
        utf8_percent_encode(&self.to_string(), NON_ALPHANUMERIC).to_string()
    }
}

impl Display for Pair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.0, self.1)
    }
}

impl FromStr for Pair {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let pair: Vec<_> = s.split('/').collect();

        if pair.len() != 2 {
            fail!(ErrorKind::Parse, "malformed trading pair: {}", s);
        }

        Ok(Pair(pair[0].parse()?, pair[1].parse()?))
    }
}

impl<'de> Deserialize<'de> for Pair {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl Serialize for Pair {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

/// Prices of currencies (internally represented as a `Decimal`)
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Price(pub Decimal);

impl Price {
    /// Create a new price from a `Decimal`
    pub(crate) fn new(decimal: Decimal) -> Result<Self, Error> {
        if decimal.to_f32().is_none() || decimal.to_f64().is_none() {
            fail!(ErrorKind::Parse, "price cannot be represented as float");
        }

        Ok(Price(decimal))
    }
}

impl Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Price {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Self::new(s.parse()?)
    }
}

impl<'de> Deserialize<'de> for Price {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl Serialize for Price {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}
