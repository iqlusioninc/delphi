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

    ///Canadian Dollar
    Cad,

    ///Swiss Franc
    Chf,

    ///Chinese Yuan
    Cny,

    /// Ethereum
    Eth,

    /// Euro
    Eur,

    /// UK Pounds
    Gbp,

    ///Indian Rupee
    Inr,

    ///Japanese Yen
    Jpy,

    /// South Korean won
    Krw,

    /// Terra Luna
    Luna,

    /// Mongolian Tugrik
    Mnt,

    /// IMF Special Drawing Rights
    Sdr,

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
            Currency::Cad => "CAD",
            Currency::Chf => "CHF",
            Currency::Eth => "ETH",
            Currency::Eur => "EUR",
            Currency::Gbp => "GBP",
            Currency::Krw => "KRW",
            Currency::Luna => "LUNA",
            Currency::Usd => "USD",
            Currency::Usdc => "USDC",
            Currency::Usdt => "USDT",
            Currency::Other(other) => other.as_ref(),
            Currency::Sdr => "SDR",
            Currency::Mnt => "MNT",
            Currency::Cny => "CNY",
            Currency::Jpy => "JPY",
            Currency::Inr => "INR",
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
            "SDR" => Currency::Sdr,
            "CNY" => Currency::Cny,
            "JPY" => Currency::Jpy,
            "INR" => Currency::Inr,
            "CAD" => Currency::Cad,
            "CHF" => Currency::Chf,
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

impl Currency {
    ///Long name for IMF csv
    pub fn imf_long_name(&self) -> String {
        match self {
            Currency::Krw => "Korean won".to_string(),
            Currency::Usd => "U.S. dollar".to_string(),
            Currency::Luna => "N/A".to_string(),
            Currency::Sdr => "SDR".to_string(),
            Currency::Other(other) => other.to_string(),
            Currency::Atom => "N/A".to_string(),
            Currency::Bnb => "N/A".to_string(),
            Currency::Bkrw => "N/A".to_string(),
            Currency::Btc => "N/A".to_string(),
            Currency::Busd => "N/A".to_string(),
            Currency::Eth => "N/A".to_string(),
            Currency::Eur => "EUR".to_string(),
            Currency::Gbp => "N/A".to_string(),
            Currency::Usdc => "N/A".to_string(),
            Currency::Usdt => "N/A".to_string(),
            Currency::Mnt => "N/A".to_string(),
            Currency::Cny => "N/A".to_string(),
            Currency::Jpy => "N/A".to_string(),
            Currency::Inr => "N/A".to_string(),
            Currency::Cad => "N/A".to_string(),
            Currency::Chf => "N/A".to_string(),
        }
    }
}
