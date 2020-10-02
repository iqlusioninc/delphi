//! Prices (wrapper for `Decimal`)

use crate::{prelude::*, Error, ErrorKind};
use rust_decimal::{prelude::*, Decimal};
use serde::{de, ser, Deserialize, Serialize};
use std::{
    cmp::Ordering,
    convert::TryFrom,
    fmt::{self, Display},
    ops::{Add, Deref, Div, Mul},
    str::FromStr,
};

/// Prices of currencies (internally represented as a `Decimal`)
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Price(Decimal);

impl Price {
    /// Create a new price from a `Decimal`
    pub(crate) fn new(decimal: Decimal) -> Result<Self, Error> {
        if decimal.to_f32().is_none() || decimal.to_f64().is_none() {
            fail!(ErrorKind::Parse, "price cannot be represented as float");
        }

        Ok(Price(decimal))
    }
}

impl Add for Price {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Mul for Price {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl Mul<Decimal> for Price {
    type Output = Self;

    fn mul(self, rhs: Decimal) -> Self {
        Self(self.0 * rhs)
    }
}

impl Div<Decimal> for Price {
    type Output = Self;

    fn div(self, rhs: Decimal) -> Price {
        Self(self.0 / rhs)
    }
}

impl Div<u64> for Price {
    type Output = Self;

    fn div(self, rhs: u64) -> Price {
        self / Decimal::from(rhs)
    }
}

impl Deref for Price {
    type Target = Decimal;

    fn deref(&self) -> &Decimal {
        &self.0
    }
}

impl Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Price {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Self::new(s.parse()?)
    }
}

impl<'de> Deserialize<'de> for Price {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use de::Error;
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl Serialize for Price {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl TryFrom<Decimal> for Price {
    type Error = Error;

    fn try_from(decimal: Decimal) -> Result<Price, Error> {
        Price::new(decimal)
    }
}

impl From<Price> for Decimal {
    fn from(price: Price) -> Decimal {
        price.0
    }
}

impl From<&Price> for Decimal {
    fn from(price: &Price) -> Decimal {
        price.0
    }
}

/// Quoted prices and quantities as sourced from the order book
#[derive(Clone, Debug, Eq)]
pub struct PriceQuantity {
    /// Price
    pub price: Price,

    /// Quantity
    pub quantity: Decimal,
}

impl Ord for PriceQuantity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price.cmp(&other.price)
    }
}

impl PartialOrd for PriceQuantity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PriceQuantity {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price
    }
}
