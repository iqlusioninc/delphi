//! Currency support (i.e. assets)

use crate::networks::terra::denom::Denom;
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

    ///Australian Dollar
    Aud,

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

    ///Danish Krone
    Dkk,

    /// Ethereum
    Eth,

    /// Euro
    Eur,

    /// UK Pounds
    Gbp,

    ///Hong Kong Dollar
    Hkd,

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

    ///Singapore Dollar
    Sgd,

    ///Thai baht
    Thb,

    /// US dollars
    Usd,

    /// Circle stablecoin
    Usdc,

    /// Tether USDT stablecoin
    Usdt,

    /// Swedish Krona
    Sek,

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
            Currency::Sdr => "XDR",
            Currency::Mnt => "MNT",
            Currency::Cny => "CNY",
            Currency::Jpy => "JPY",
            Currency::Inr => "INR",
            Currency::Hkd => "HKD",
            Currency::Aud => "AUD",
            Currency::Sgd => "SGD",
            Currency::Thb => "THB",
            Currency::Sek => "SEK",
            Currency::Dkk => "DKK",
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
            "XDR" => Currency::Sdr,
            "CNY" => Currency::Cny,
            "JPY" => Currency::Jpy,
            "INR" => Currency::Inr,
            "CAD" => Currency::Cad,
            "CHF" => Currency::Chf,
            "HKD" => Currency::Hkd,
            "AUD" => Currency::Aud,
            "SGD" => Currency::Sgd,
            "THB" => Currency::Thb,
            "SEK" => Currency::Sek,
            "DKK" => Currency::Dkk,
            other => Currency::Other(other.to_owned()),
        })
    }
}

impl From<Denom> for Currency {
    fn from(denom: Denom) -> Currency {
        match denom {
            Denom::Ueur => Currency::Eur,
            Denom::Ucny => Currency::Cny,
            Denom::Ujpy => Currency::Jpy,
            Denom::Ugbp => Currency::Gbp,
            Denom::Uinr => Currency::Inr,
            Denom::Ucad => Currency::Cad,
            Denom::Uchf => Currency::Chf,
            Denom::Uhkd => Currency::Hkd,
            Denom::Uaud => Currency::Aud,
            Denom::Usgd => Currency::Sgd,
            Denom::Ukrw => Currency::Krw,
            Denom::Umnt => Currency::Mnt,
            Denom::Usdr => Currency::Sdr,
            Denom::Uusd => Currency::Usd,
            Denom::Uthb => Currency::Thb,
            Denom::Usek => Currency::Sek,
            Denom::Udkk => Currency::Dkk,
        }
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
            Currency::Sdr => "XDR".to_string(),
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
            Currency::Hkd => "N/A".to_string(),
            Currency::Aud => "N/A".to_string(),
            Currency::Sgd => "N/A".to_string(),
            Currency::Thb => "N/A".to_string(),
            Currency::Sek => "N/A".to_string(),
            Currency::Dkk => "N/A".to_string(),
        }
    }
}
