//! Exchange rate denominations

use crate::{
    currency::Currency,
    error::{Error, ErrorKind},
    prelude::*,
    sources::Sources,
    trading_pair::TradingPair,
};
use rust_decimal::Decimal;
use serde::{de, ser, Deserialize, Serialize};
use std::{
    convert::TryInto,
    fmt::{self, Display},
    str::FromStr,
};
use tokio::try_join;

/// Denomination
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Denom {
    /// Korean Wan
    Ukrw,

    /// Mongolian Tugrik
    Umnt,

    /// IMF Special Drawing Rights
    Usdr,

    /// US Dollars
    Uusd,

    /// Euro
    Ueur,

    /// Chinese Yuan
    Ucny,

    /// Japanese Yen
    Ujpy,

    /// UK Pound
    Ugbp,

    ///Indian Rupee
    Uinr,

    ///Canadian Dollar
    Ucad,

    ///Swiss Franc
    Uchf,

    ///Hong Kong Dollar
    Uhkd,

    ///Australian Dollar
    Uaud,

    ///Singapore Dollar
    Usgd,

    ///Thai baht
    Uthb,

    ///Swedish Krona
    Usek,

    ///Danish Krone
    Udkk,

    /// Norwegian Krone
    Unok,

    ///Taiwan Dollar
    Utwd,
}

impl Denom {
    /// Get a slice of the [`Denom`] kinds
    pub fn kinds() -> &'static [Denom] {
        &[
            Denom::Ukrw,
            Denom::Umnt,
            Denom::Usdr,
            Denom::Uusd,
            Denom::Ueur,
            Denom::Ucny,
            Denom::Ujpy,
            Denom::Ugbp,
            Denom::Uinr,
            Denom::Ucad,
            Denom::Uchf,
            Denom::Uhkd,
            Denom::Uaud,
            Denom::Usgd,
            Denom::Uthb,
            Denom::Usek,
            Denom::Udkk,
            Denom::Unok,
            Denom::Utwd,
        ]
    }

    /// Get the code corresponding to a [`Denom`]
    pub fn as_str(self) -> &'static str {
        match self {
            Denom::Ukrw => "ukrw",
            Denom::Umnt => "umnt",
            Denom::Usdr => "usdr",
            Denom::Uusd => "uusd",
            Denom::Ueur => "ueur",
            Denom::Ucny => "ucny",
            Denom::Ujpy => "ujpy",
            Denom::Ugbp => "ugbp",
            Denom::Uinr => "uinr",
            Denom::Ucad => "ucad",
            Denom::Uchf => "uchf",
            Denom::Uhkd => "uhkd",
            Denom::Uaud => "uaud",
            Denom::Usgd => "usgd",
            Denom::Uthb => "uthb",
            Denom::Usek => "usek",
            Denom::Udkk => "udkk",
            Denom::Unok => "unok",
            Denom::Utwd => "utwd",
        }
    }

    /// Get the exchange rate for this [`Denom`]
    pub async fn get_exchange_rate(self, sources: &Sources) -> Result<stdtx::Decimal, Error> {
        match self {
            Denom::Ukrw => {
                let bithumb_response = sources
                    .bithumb
                    .trading_pairs(&TradingPair(Currency::Luna, Currency::Krw))
                    .await?;

                let mut luna_krw: Decimal = bithumb_response.into();

                luna_krw.rescale(18);
                Ok(luna_krw.try_into()?)
            }

            Denom::Umnt => {
                let (
                    currencylayer_response_usd,
                    currencylayer_response_krw,
                    binance_response,
                    coinone_midpoint,
                ) = try_join!(
                    sources
                        .currencylayer
                        .trading_pairs(&TradingPair(Currency::Usd, Currency::Mnt)),
                    sources
                        .currencylayer
                        .trading_pairs(&TradingPair(Currency::Krw, Currency::Mnt)),
                    sources
                        .binance
                        .approx_price_for_pair(&TradingPair(Currency::Luna, Currency::Usd)),
                    sources
                        .coinone
                        .trading_pairs(&TradingPair(Currency::Luna, Currency::Krw))
                )?;

                let mut luna_mnt = Decimal::from(
                    (binance_response * currencylayer_response_usd
                        + coinone_midpoint * currencylayer_response_krw)
                        / 2,
                );

                luna_mnt.rescale(18);
                Ok(luna_mnt.try_into()?)
            }

            Denom::Uusd => {
                let binance_response = sources
                    .binance
                    .approx_price_for_pair(&TradingPair(Currency::Luna, Currency::Usd))
                    .await?;

                let mut luna_usd: Decimal = binance_response.into();
                luna_usd.rescale(18);
                Ok(luna_usd.try_into()?)
            }

            Denom::Usdr => {
                let (imf_sdr_response, coinone_midpoint) = try_join!(
                    sources
                        .imf_sdr
                        .trading_pairs(&TradingPair(Currency::Krw, Currency::Sdr)),
                    sources
                        .coinone
                        .trading_pairs(&TradingPair(Currency::Luna, Currency::Krw))
                )?;

                let mut luna_sdr = Decimal::from(coinone_midpoint * imf_sdr_response.price);
                luna_sdr.rescale(18);
                Ok(luna_sdr.try_into()?)
            }

            _ => luna_rate_via_usd(sources, self.into()).await,
        }
    }
}

async fn luna_rate_via_usd(sources: &Sources, cur: Currency) -> Result<stdtx::Decimal, Error> {
    let pair_1 = TradingPair(Currency::Usd, cur);

    let (currencylayer_response_usd, binance_response) = try_join!(
        sources.currencylayer.trading_pairs(&pair_1),
        sources
            .binance
            .approx_price_for_pair(&TradingPair(Currency::Luna, Currency::Usd)),
    )?;

    let mut luna_cur = Decimal::from(binance_response * currencylayer_response_usd);

    luna_cur.rescale(18);
    Ok(luna_cur.try_into()?)
}

impl Display for Denom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Denom {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_ascii_lowercase().as_ref() {
            "ukrw" => Ok(Denom::Ukrw),
            "umnt" => Ok(Denom::Umnt),
            "usdr" => Ok(Denom::Usdr),
            "uusd" => Ok(Denom::Uusd),
            "ueur" => Ok(Denom::Ueur),
            "ucny" => Ok(Denom::Ucny),
            "ujpy" => Ok(Denom::Ujpy),
            "ugbp" => Ok(Denom::Ugbp),
            "uinr" => Ok(Denom::Uinr),
            "ucad" => Ok(Denom::Ucad),
            "uchf" => Ok(Denom::Uchf),
            "uthb" => Ok(Denom::Uthb),
            "usek" => Ok(Denom::Usek),
            "udkk" => Ok(Denom::Udkk),
            "unok" => Ok(Denom::Unok),
            "utwd" => Ok(Denom::Utwd),

            _ => fail!(ErrorKind::Currency, "unknown Terra denom: {}", s),
        }
    }
}

impl<'de> Deserialize<'de> for Denom {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use de::Error;
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(D::Error::custom)
    }
}

impl Serialize for Denom {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}
