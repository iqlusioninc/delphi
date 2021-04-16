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
        ]
    }

    /// Get the code corresponding to a [`Denom`]
    pub fn as_str(self) -> &'static str {
        match self {
            Denom::Ukrw => "Ukrw",
            Denom::Umnt => "Umnt",
            Denom::Usdr => "Usdr",
            Denom::Uusd => "Uusd",
            Denom::Ueur => "Ueur",
            Denom::Ucny => "Ucny",
            Denom::Ujpy => "Ujpy",
            Denom::Ugbp => "Ugbp",
            Denom::Uinr => "Uinr",
            Denom::Ucad => "Ucad",
            Denom::Uchf => "Uchf",
            Denom::Uhkd => "Uhkd",
            Denom::Uaud => "Uaud",
            Denom::Usgd => "Usgd",
            Denom::Uthb => "Uthb",
        }
    }

    /// Get the exchange rate for this [`Denom`]
    pub async fn get_exchange_rate(self, sources: &Sources) -> Result<stdtx::Decimal, Error> {
        match self {
            Denom::Ukrw => {
                let pair = TradingPair(Currency::Luna, Currency::Krw);

                let (coinone_midpoint, gdac_midpoint, binance_response) = try_join!(
                    sources.coinone.trading_pairs(&pair),
                    sources.gdac.trading_pairs(&pair),
                    sources.binance.approx_price_for_pair(&pair)
                )?;

                //Midpoint avg for all sources
                let mut luna_krw =
                    Decimal::from((coinone_midpoint + gdac_midpoint + binance_response) / 3);

                luna_krw.rescale(18);
                Ok(luna_krw.try_into()?)
            }

            Denom::Umnt => {
                let (
                    alphavantage_response_usd,
                    alphavantage_response_krw,
                    binance_response,
                    coinone_midpoint,
                ) = try_join!(
                    sources
                        .alphavantage
                        .trading_pairs(&TradingPair(Currency::Usd, Currency::Mnt)),
                    sources
                        .alphavantage
                        .trading_pairs(&TradingPair(Currency::Krw, Currency::Mnt)),
                    sources
                        .binance
                        .approx_price_for_pair(&TradingPair(Currency::Luna, Currency::Usd)),
                    sources
                        .coinone
                        .trading_pairs(&TradingPair(Currency::Luna, Currency::Krw))
                )?;

                let mut luna_mnt = Decimal::from(
                    (binance_response * alphavantage_response_usd
                        + coinone_midpoint * alphavantage_response_krw)
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

    let (alphavantage_response_usd, binance_response) = try_join!(
        sources.alphavantage.trading_pairs(&pair_1),
        sources
            .binance
            .approx_price_for_pair(&TradingPair(Currency::Luna, Currency::Usd)),
    )?;

    let mut luna_cur = Decimal::from(binance_response * alphavantage_response_usd);

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
            "Ukrw" => Ok(Denom::Ukrw),
            "Umnt" => Ok(Denom::Umnt),
            "Usdr" => Ok(Denom::Usdr),
            "Uusd" => Ok(Denom::Uusd),
            "Ueur" => Ok(Denom::Ueur),
            "Ucny" => Ok(Denom::Ucny),
            "Ujpy" => Ok(Denom::Ujpy),
            "Ugbp" => Ok(Denom::Ugbp),
            "Uinr" => Ok(Denom::Uinr),
            "Ucad" => Ok(Denom::Ucad),
            "Uchf" => Ok(Denom::Uchf),
            "Uthb" => Ok(Denom::Uthb),

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
