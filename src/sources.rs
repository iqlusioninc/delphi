//! Data sources

pub mod coinone;
pub mod gdac;

use crate::{
    error::{Error, ErrorKind},
    prelude::*,
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{de, Deserialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

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
        use de::Error;
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(D::Error::custom)
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
        use de::Error;
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(D::Error::custom)
    }
}
