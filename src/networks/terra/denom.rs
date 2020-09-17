//! Exchange rate denominations

use crate::error::Error;
use crate::sources::gdac::GdacSource;
use crate::sources::gopax::GopaxSource;
use crate::sources::{coinone::CoinoneSource, Currency, Pair, Price};
use crate::sources::{weighted_avg_ask, weighted_avg_bid};
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
    pub async fn get_exchange_rate(self) -> Result<stdtx::Decimal, Error> {
        match self {
            Denom::UKRW => {
                // Source: CoinOne
                let coinone_response = CoinoneSource::new()
                    .trading_pairs(&Pair(Currency::Luna, Currency::Krw))
                    .await?;
                dbg!(&coinone_response);
                let ask_weighted_avg = weighted_avg_ask(&coinone_response)?;
                dbg!(&ask_weighted_avg);
                let bid_weighted_avg = weighted_avg_bid(&coinone_response)?;
                dbg!(&bid_weighted_avg);

                // Source: GDAC
                let gdac_response = GdacSource::new()
                    .trading_pairs(&Pair(Currency::Luna, Currency::Krw))
                    .await?;
                dbg!(&gdac_response);
                let gdac_ask_weighted_avg = weighted_avg_ask(&gdac_response)?;
                dbg!(&gdac_ask_weighted_avg);
                let gdac_bid_weighted_avg = weighted_avg_bid(&gdac_response)?;
                dbg!(&gdac_bid_weighted_avg);

                // Source: GOPAX
                let gopax_response = GopaxSource::new()
                    .trading_pairs(&Pair(Currency::Luna, Currency::Krw))
                    .await?;
                dbg!(&gopax_response);
                let gopax_ask_weighted_avg = weighted_avg_ask(&gopax_response)?;
                dbg!(&gopax_ask_weighted_avg);
                let gopax_bid_weighted_avg = weighted_avg_bid(&gopax_response)?;
                dbg!(&gopax_bid_weighted_avg);

                // Weighted avgs for all sources
                let coinone_weighted_avg =
                    Price::new((ask_weighted_avg.0 + bid_weighted_avg.0) / Decimal::new(2, 0))?;
                dbg!(&coinone_weighted_avg);
                let gdac_weighted_avg = Price::new(
                    (gdac_ask_weighted_avg.0 + gdac_bid_weighted_avg.0) / Decimal::new(2, 0),
                )?;
                dbg!(&gdac_weighted_avg);
                let gopax_weighted_avg = Price::new(
                    (gopax_ask_weighted_avg.0 + gopax_bid_weighted_avg.0) / Decimal::new(2, 0),
                )?;
                dbg!(&gopax_weighted_avg);

                let coinone_gdac_weighted_avg = Price::new(
                    (coinone_weighted_avg.0 + gdac_weighted_avg.0) / Decimal::new(2, 0),
                )?;
                dbg!(&coinone_gdac_weighted_avg);

                // TODO: debug gopax source, reporting weird prices
                // let weighted_avg = Price::new(
                //     (&coinone_weighted_avg.0 + &gdac_weighted_avg.0 + &gopax_weighted_avg.0)
                //         / Decimal::new(3, 0),
                // )?;
                // dbg!(&weighted_avg);

                let weighted_avg = coinone_gdac_weighted_avg.0;
                dbg!(&weighted_avg, weighted_avg.scale());
                let mut weighted_avg_scaled =
                    weighted_avg * Decimal::new(1, weighted_avg.scale() - 16);
                weighted_avg_scaled
                    .set_scale(18)
                    .expect("couldn't set scale");
                dbg!(&weighted_avg_scaled, weighted_avg_scaled.scale());
                Ok(stdtx::Decimal::try_from(weighted_avg_scaled)?)
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
