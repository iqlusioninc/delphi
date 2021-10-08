//! Terra message types

use super::Denom;
use crate::{
    error::{Error, ErrorKind},
    map,
    prelude::*,
    Map,
};
use rust_decimal::Decimal;
use std::fmt::{self, Display};

/// Exchange rates
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExchangeRates(Map<Denom, Decimal>);

impl ExchangeRates {
    /// Create a new set of exchange rates
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new set of exchange rates from an iterator over
    /// `(Denom,Decimal)` tuples
    // NOTE: can't use `FromIterator` here because of the `Result`
    pub fn from_exchange_rates<'a, I>(iter: I) -> Result<Self, Error>
    where
        I: Iterator<Item = &'a (Denom, Decimal)>,
    {
        let mut exchange_rates = ExchangeRates::new();

        for &(denom, rate) in iter {
            exchange_rates.add(denom, rate)?;
        }

        Ok(exchange_rates)
    }

    /// Add an exchange rate
    pub fn add(&mut self, denom: Denom, rate: Decimal) -> Result<(), Error> {
        let duplicate = self.0.insert(denom, rate).is_some();

        ensure!(
            !duplicate,
            ErrorKind::Currency,
            "duplicate exchange rate for denom: {}",
            denom
        );

        Ok(())
    }

    /// Iterate over the exchange rates
    pub fn iter(&self) -> map::Iter<'_, Denom, Decimal> {
        self.0.iter()
    }
}

impl Display for ExchangeRates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, (denom, rate)) in self.0.iter().enumerate() {
            write!(f, "{}{}", rate, denom)?;

            if i < self.0.len() - 1 {
                write!(f, ",")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Denom, ExchangeRates};

    #[test]
    fn exchange_rate_to_string() {
        let exchange_rates = ExchangeRates::from_exchange_rates(
            [
                (Denom::Uusd, "1".parse().unwrap()),
                (Denom::Usdr, "1".parse().unwrap()),
                (Denom::Umnt, "888".parse().unwrap()),
                (Denom::Ukrw, "362".parse().unwrap()),
            ]
            .iter(),
        )
        .unwrap();

        let serialized_rates = exchange_rates.to_string();
        assert_eq!(
            &serialized_rates,
            "362.000000000000000000ukrw,\
            888.000000000000000000umnt,\
            1.000000000000000000usdr,\
            1.000000000000000000uusd"
        );
    }
}
