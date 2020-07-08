//! Exchange rate denominations

use crate::error::Error;
use crate::sources::gdac::GdacSource;
use crate::sources::gopax::GopaxSource;
use crate::sources::{coinone, gdac, gopax};
use crate::sources::{coinone::CoinoneSource, Currency, Pair, Price};
use rust_decimal::Decimal;
use std::fmt::{self, Display};
use std::str::FromStr;

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
    pub async fn get_exchange_rate(self) -> Result<Decimal, Error> {
        match self {
            Denom::UKRW => {
                // Source: CoinOne
                let coinone_response = CoinoneSource::new()
                    .trading_pairs(&Pair(Currency::Luna, Currency::Krw))
                    .await?;
                dbg!(&coinone_response);
                let ask_weighted_avg = get_ask_weighted_average(&coinone_response);
                dbg!(&ask_weighted_avg);
                let bid_weighted_avg = get_bid_weighted_average(&coinone_response);
                dbg!(&bid_weighted_avg);

                // Source: GDAC
                let gdac_response = GdacSource::new()
                    .trading_pairs(&Pair(Currency::Luna, Currency::Krw))
                    .await?;
                dbg!(&gdac_response);
                let gdac_ask_weighted_avg = gdac_get_ask_weighted_average(&gdac_response);
                dbg!(&gdac_ask_weighted_avg);
                let gdac_bid_weighted_avg = gdac_get_bid_weighted_average(&gdac_response);
                dbg!(&gdac_bid_weighted_avg);

                // Source: GOPAX
                let gopax_response = GopaxSource::new()
                    .trading_pairs(&Pair(Currency::Luna, Currency::Krw))
                    .await?;
                dbg!(&gopax_response);
                let gopax_ask_weighted_avg = gopax_get_ask_weighted_average(&gopax_response);
                dbg!(&gopax_ask_weighted_avg);
                let gopax_bid_weighted_avg = gopax_get_bid_weighted_average(&gopax_response);
                dbg!(&gopax_bid_weighted_avg);

                // Weighted avgs for all sources
                let coinone_weighted_avg =
                    Price::new((ask_weighted_avg?.0 + bid_weighted_avg?.0) / Decimal::new(2, 0));
                dbg!(&coinone_weighted_avg);
                let gdac_weighted_avg = Price::new(
                    (gdac_ask_weighted_avg?.0 + gdac_bid_weighted_avg?.0) / Decimal::new(2, 0),
                );
                dbg!(&gdac_weighted_avg);
                let gopax_weighted_avg = Price::new(
                    (gopax_ask_weighted_avg?.0 + gopax_bid_weighted_avg?.0) / Decimal::new(2, 0),
                );
                dbg!(&gopax_weighted_avg);

                Ok(Decimal::from(-1i8))
            }
            _ => Ok(Decimal::from(-1i8)),
        }
    }
}

impl Display for Denom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Ask price weighted average
pub fn get_ask_weighted_average(response: &coinone::Response) -> Result<Price, Error> {
    let mut price_sum_product = Decimal::from(0u8);
    let mut total = Decimal::from(0u8);
    for ask in &response.ask {
        let quantity = Decimal::from_str(&ask.qty.clone())?;
        price_sum_product += ask.price.0 * quantity;
        total += quantity;
    }

    let weighted_avg = Price::new(price_sum_product / total)?;
    Ok(weighted_avg)
}

/// Bid price weighted average
pub fn get_bid_weighted_average(response: &coinone::Response) -> Result<Price, Error> {
    let mut price_sum_product = Decimal::from(0u8);
    let mut total = Decimal::from(0u8);
    for bid in &response.bid {
        let quantity = Decimal::from_str(&bid.qty.clone())?;
        price_sum_product += bid.price.0 * quantity;
        total += quantity;
    }

    let weighted_avg = Price::new(price_sum_product / total)?;
    Ok(weighted_avg)
}

/// GDAC Ask price weighted average
pub fn gdac_get_ask_weighted_average(response: &gdac::Quote) -> Result<Price, Error> {
    let mut price_sum_product = Decimal::from(0u8);
    let mut total = Decimal::from(0u8);
    for ask in &response.ask {
        let quantity = Decimal::from_str(&ask.volume.clone())?;
        price_sum_product += ask.price.0 * quantity;
        total += quantity;
    }

    let weighted_avg = Price::new(price_sum_product / total)?;
    Ok(weighted_avg)
}

/// GDAC Bid price weighted average
pub fn gdac_get_bid_weighted_average(response: &gdac::Quote) -> Result<Price, Error> {
    let mut price_sum_product = Decimal::from(0u8);
    let mut total = Decimal::from(0u8);
    for bid in &response.bid {
        let quantity = Decimal::from_str(&bid.volume.clone())?;
        price_sum_product += bid.price.0 * quantity;
        total += quantity;
    }

    let weighted_avg = Price::new(price_sum_product / total)?;
    Ok(weighted_avg)
}

/// GOPAX Ask price weighted average
pub fn gopax_get_ask_weighted_average(response: &gopax::Quote) -> Result<Price, Error> {
    // id, price, volume
    let mut price_sum_product = Decimal::from(0u8);
    let mut total = Decimal::from(0u8);
    for ask in &response.ask {
        let quantity = ask.volume;
        price_sum_product += ask.price * quantity;
        total += quantity;
    }

    let weighted_avg = Price::new(price_sum_product / total)?;
    Ok(weighted_avg)
}

/// GOPAX Bid price weighted average
pub fn gopax_get_bid_weighted_average(response: &gopax::Quote) -> Result<Price, Error> {
    let mut price_sum_product = Decimal::from(0u8);
    let mut total = Decimal::from(0u8);
    for bid in &response.bid {
        let quantity = bid.volume;
        price_sum_product += bid.price * quantity;
        total += quantity;
    }

    let weighted_avg = Price::new(price_sum_product / total)?;
    Ok(weighted_avg)
}
