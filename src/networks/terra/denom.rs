//! Exchange rate denominations

use crate::config::AlphavantageConfig;
use crate::error::Error;
use crate::sources::gdac::GdacSource;
use crate::sources::midpoint;
use crate::sources::{alphavantage::AlphavantageSource, coinone::CoinoneSource, Currency, Pair};
use rust_decimal::Decimal;
use std::convert::TryFrom;
use std::fmt::{self, Display};

/// Denomination
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Denom {
    /// Korean Wan
    UKRW,

    /// Singaporean Dollar
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
    pub async fn get_exchange_rate(
        self,
        alphavantage_config: AlphavantageConfig,
    ) -> Result<stdtx::Decimal, Error> {
        match self {
            Denom::UKRW => {
                // Source: CoinOne
                let coinone_response = CoinoneSource::new()
                    .trading_pairs(&Pair(Currency::Luna, Currency::Krw))
                    .await?;
                // dbg!(&coinone_response);
                let coinone_midpoint = midpoint(&coinone_response)?;
                dbg!(&coinone_midpoint);

                // Source: GDAC
                let gdac_response = GdacSource::new()
                    .trading_pairs(&Pair(Currency::Luna, Currency::Krw))
                    .await?;
                // dbg!(&gdac_response);
                let gdac_midpoint = midpoint(&gdac_response)?;
                dbg!(&gdac_midpoint);

                //Midpoint avg for all sources
                let mut midpoint_avg = (coinone_midpoint.0 + gdac_midpoint.0) / Decimal::from(2);
                dbg!(&midpoint_avg);

                dbg!(&midpoint_avg, midpoint_avg.scale());
                midpoint_avg.rescale(18);
                dbg!(&midpoint_avg, midpoint_avg.scale());
                Ok(stdtx::Decimal::try_from(midpoint_avg)?)
            }

            Denom::UMNT => {
                // Source: AlphaVantage
                let alphavantage_response = AlphavantageSource::new(alphavantage_config.apikey)
                    .trading_pairs(&Pair(Currency::Krw, Currency::Other("SGD".to_owned())))
                    .await?;

                // Source: CoinOne
                let coinone_response = CoinoneSource::new()
                    .trading_pairs(&Pair(Currency::Luna, Currency::Krw))
                    .await?;
                // dbg!(&coinone_response);
                let coinone_midpoint = midpoint(&coinone_response)?;

                let krw_sgd = coinone_midpoint.0
                    * alphavantage_response
                        .realtime_currency_exchange_rate
                        .exchange_rate
                        .0;
                dbg!(krw_sgd);

                Ok(stdtx::Decimal::try_from(krw_sgd)?)
            }

            _ => Ok(stdtx::Decimal::from(-1i8)),
        }
    }
}

impl Display for Denom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
