//! Data sources

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
    fmt::{self, Display},
    str::FromStr,
};
use async_trait::async_trait;


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
pub struct Price(Decimal);

impl Price {
    /// Create a new price from a `Decimal`
    pub(crate) fn new(decimal: Decimal) -> Result<Self, Error> {
        if decimal.to_f32().is_none() || decimal.to_f64().is_none() {
            fail!(ErrorKind::Parse, "price cannot be represented as float");
        }

        Ok(Price(decimal))
    }

    /// Convert price to `f32`
    pub fn to_f32(&self) -> f32 {
        // This is guaranteed to always be `Some` by `new`
        self.0.to_f32().unwrap()
    }

    /// Convert prices to `f64`
    pub fn to_f64(&self) -> f64 {
        // This is guaranteed to always be `Some` by `new`
        self.0.to_f64().unwrap()
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

#[async_trait]
pub(crate) trait ComputablePrice {
    async fn run_price(&self,pair:Pair )-> Result<Price,Error>;
}
