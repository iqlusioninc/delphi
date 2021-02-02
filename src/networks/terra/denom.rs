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
    UKRW,

    /// Mongolian Tugrik
    UMNT,

    /// IMF Special Drawing Rights
    USDR,

    /// US Dollars
    UUSD,

    /// Euro
    UEUR,

    /// Chinese Yuan
    UCNY,

    /// Japanese Yen
    UJPY,

    /// UK Pound
    UGBP,
}

impl Denom {
    /// Get a slice of the [`Denom`] kinds
    pub fn kinds() -> &'static [Denom] {
        &[
            Denom::UKRW,
            Denom::UMNT,
            Denom::USDR,
            Denom::UUSD,
            Denom::UEUR,
            Denom::UCNY,
            Denom::UJPY,
            Denom::UGBP,
        ]
    }

    /// Get the code corresponding to a [`Denom`]
    pub fn as_str(self) -> &'static str {
        match self {
            Denom::UKRW => "ukrw",
            Denom::UMNT => "umnt",
            Denom::USDR => "usdr",
            Denom::UUSD => "uusd",
            Denom::UEUR => "ueur",
            Denom::UCNY => "ucny",
            Denom::UJPY => "ujpy",
            Denom::UGBP => "ugbp",
        }
    }

    /// Get the exchange rate for this [`Denom`]
    pub async fn get_exchange_rate(self, sources: &Sources) -> Result<stdtx::Decimal, Error> {
        match self {
            Denom::UKRW => {
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

            Denom::UMNT => {
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

            Denom::UUSD => {
                let binance_response = sources
                    .binance
                    .approx_price_for_pair(&TradingPair(Currency::Luna, Currency::Usd))
                    .await?;

                let mut luna_usd: Decimal = binance_response.into();
                luna_usd.rescale(18);
                Ok(luna_usd.try_into()?)
            }

            Denom::USDR => {
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

            Denom::UEUR => {
                let (alphavantage_response_usd, binance_response) = try_join!(
                    sources
                        .alphavantage
                        .trading_pairs(&TradingPair(Currency::Usd, Currency::Eur)),
                    sources
                        .binance
                        .approx_price_for_pair(&TradingPair(Currency::Luna, Currency::Usd)),
                )?;

                let mut luna_eur = Decimal::from(binance_response * alphavantage_response_usd);

                luna_eur.rescale(18);
                Ok(luna_eur.try_into()?)
            }

            Denom::UCNY => {
                let (alphavantage_response_usd, binance_response) = try_join!(
                    sources
                        .alphavantage
                        .trading_pairs(&TradingPair(Currency::Usd, Currency::Cny)),
                    sources
                        .binance
                        .approx_price_for_pair(&TradingPair(Currency::Luna, Currency::Usd)),
                )?;

                let mut luna_cny = Decimal::from(binance_response * alphavantage_response_usd);

                luna_cny.rescale(18);
                Ok(luna_cny.try_into()?)
            }

            Denom::UJPY => {
                let (alphavantage_response_usd, binance_response) = try_join!(
                    sources
                        .alphavantage
                        .trading_pairs(&TradingPair(Currency::Usd, Currency::Jpy)),
                    sources
                        .binance
                        .approx_price_for_pair(&TradingPair(Currency::Luna, Currency::Usd)),
                )?;

                let mut luna_jpy = Decimal::from(binance_response * alphavantage_response_usd);

                luna_jpy.rescale(18);
                Ok(luna_jpy.try_into()?)
            }

            Denom::UGBP => {
                let (alphavantage_response_usd, binance_response) = try_join!(
                    sources
                        .alphavantage
                        .trading_pairs(&TradingPair(Currency::Usd, Currency::Gbp)),
                    sources
                        .binance
                        .approx_price_for_pair(&TradingPair(Currency::Luna, Currency::Usd)),
                )?;

                let mut luna_gbp = Decimal::from(binance_response * alphavantage_response_usd);

                luna_gbp.rescale(18);
                Ok(luna_gbp.try_into()?)
            }
        }
    }
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
            "ukrw" => Ok(Denom::UKRW),
            "umnt" => Ok(Denom::UMNT),
            "usdr" => Ok(Denom::USDR),
            "uusd" => Ok(Denom::UUSD),
            "ueur" => Ok(Denom::UEUR),
            "ucny" => Ok(Denom::UCNY),
            "ujpy" => Ok(Denom::UJPY),
            "ugbp" => Ok(Denom::UGBP),
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
