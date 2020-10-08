//! Exchange rate denominations

use crate::{
    currency::Currency,
    error::{Error, ErrorKind},
    prelude::*,
    sources::{midpoint, Sources},
    trading_pair::TradingPair,
};
use rust_decimal::Decimal;
use serde::{de, ser, Deserialize, Serialize};
use std::{
    convert::TryInto,
    fmt::{self, Display},
    str::FromStr,
};

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
}

impl Denom {
    /// Get a slice of the [`Denom`] kinds
    pub fn kinds() -> &'static [Denom] {
        &[Denom::UKRW, Denom::UMNT, Denom::USDR, Denom::UUSD]
    }

    /// Get the code corresponding to a [`Denom`]
    pub fn as_str(self) -> &'static str {
        match self {
            Denom::UKRW => "ukrw",
            Denom::UMNT => "umnt",
            Denom::USDR => "usdr",
            Denom::UUSD => "uusd",
        }
    }

    /// Get the exchange rate for this [`Denom`]
    pub async fn get_exchange_rate(self, sources: &Sources) -> Result<stdtx::Decimal, Error> {
        match self {
            Denom::UKRW => {
                // Source: CoinOne
                let coinone_midpoint = sources
                    .coinone
                    .trading_pairs(&TradingPair(Currency::Luna, Currency::Krw))
                    .await?;
                dbg!(&coinone_midpoint);

                // Source: GDAC
                let gdac_response = sources
                    .gdac
                    .trading_pairs(&TradingPair(Currency::Luna, Currency::Krw))
                    .await?;
                // dbg!(&gdac_response);
                let gdac_midpoint = midpoint(&gdac_response)?;
                dbg!(&gdac_midpoint);

                // Source: Binance
                let binance_response = sources
                    .binance
                    .approx_price_for_pair(&"LUNA/KRW".parse().unwrap())
                    .await
                    .unwrap();

                dbg!(&binance_response);

                //Midpoint avg for all sources
                let mut luna_krw =
                    Decimal::from((coinone_midpoint + gdac_midpoint + binance_response) / 3);
                dbg!(&luna_krw);

                dbg!(&luna_krw, luna_krw.scale());
                luna_krw.rescale(18);
                dbg!(&luna_krw, luna_krw.scale());
                Ok(luna_krw.try_into()?)
            }

            Denom::UMNT => {
                // Source: AlphaVantage
                let alphavantage_response_usd = sources
                    .alphavantage
                    .trading_pairs(&TradingPair(Currency::Usd, Currency::Mnt))
                    .await?;

                let alphavantage_response_krw = sources
                    .alphavantage
                    .trading_pairs(&TradingPair(Currency::Krw, Currency::Mnt))
                    .await?;

                // Source: Binance
                let binance_response = sources
                    .binance
                    .approx_price_for_pair(&"LUNA/USD".parse().unwrap())
                    .await
                    .unwrap();

                // Source: CoinOne
                let coinone_midpoint = sources
                    .coinone
                    .trading_pairs(&TradingPair(Currency::Luna, Currency::Krw))
                    .await?;

                let mut luna_mnt = Decimal::from(
                    (binance_response
                        * alphavantage_response_usd
                            .realtime_currency_exchange_rate
                            .exchange_rate
                        + coinone_midpoint
                            * alphavantage_response_krw
                                .realtime_currency_exchange_rate
                                .exchange_rate)
                        / 2,
                );
                dbg!(luna_mnt);

                luna_mnt.rescale(18);

                Ok(luna_mnt.try_into()?)
            }

            Denom::UUSD => {
                // Source: Binance
                let binance_response = sources
                    .binance
                    .approx_price_for_pair(&"LUNA/USD".parse().unwrap())
                    .await
                    .unwrap();

                let mut luna_usd: Decimal = binance_response.into();

                dbg!(luna_usd);

                luna_usd.rescale(18);

                Ok(luna_usd.try_into()?)
            }

            Denom::USDR => {
                //Source IMF_SDR
                let imf_sdr_response = sources
                    .imf_sdr
                    .trading_pairs(&TradingPair(Currency::Krw, Currency::Sdr))
                    .await
                    .unwrap();

                // Source: CoinOne
                let coinone_midpoint = sources
                    .coinone
                    .trading_pairs(&TradingPair(Currency::Luna, Currency::Krw))
                    .await?;

                let mut luna_sdr = Decimal::from(coinone_midpoint * imf_sdr_response.price);

                dbg!(luna_sdr);

                luna_sdr.rescale(18);

                Ok(luna_sdr.try_into()?)
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
