//! Currency support (i.e. assets)

use crate::Error;
use serde::{de, ser, Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

/// Currencies for use in trading pairs
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Currency {
    /// Cosmos Atom
    Atom,

    /// Binance Coin
    Bnb,

    /// Binance KRW (won) stablecoin
    Bkrw,

    /// Bitcoin
    Btc,

    /// Binance USD stablecoin
    Busd,

    /// Ethereum
    Eth,

    /// Euro
    Eur,

    /// UK Pounds
    Gbp,

    /// South Korean won
    Krw,

    /// Terra Luna
    Luna,

    /// US dollars
    Usd,

    /// Circle stablecoin
    Usdc,

    /// Tether USDT stablecoin
    Usdt,

    /// Other (open-ended)
    Other(String),
}

impl Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Currency::Atom => "ATOM",
            Currency::Bnb => "BNB",
            Currency::Bkrw => "BKRW",
            Currency::Btc => "BTC",
            Currency::Busd => "BUSD",
            Currency::Eth => "ETH",
            Currency::Eur => "EUR",
            Currency::Gbp => "GBP",
            Currency::Krw => "KRW",
            Currency::Luna => "LUNA",
            Currency::Usd => "USD",
            Currency::Usdc => "USDC",
            Currency::Usdt => "USDT",
            Currency::Other(other) => other.as_ref(),
        })
    }
}

impl FromStr for Currency {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(match s.to_ascii_uppercase().as_ref() {
            "ATOM" => Currency::Atom,
            "BNB" => Currency::Bnb,
            "BKRW" => Currency::Bkrw,
            "BTC" => Currency::Btc,
            "BUSD" => Currency::Busd,
            "ETH" => Currency::Eth,
            "EUR" => Currency::Eur,
            "GBP" => Currency::Gbp,
            "KRW" => Currency::Krw,
            "LUNA" => Currency::Luna,
            "USD" => Currency::Usd,
            "USDC" => Currency::Usdc,
            "USDT" => Currency::Usdt,
            other => Currency::Other(other.to_owned()),
        })
    }
}

impl<'de> Deserialize<'de> for Currency {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use de::Error;
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
