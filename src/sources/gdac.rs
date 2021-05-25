//! GDAC Source Provider (v0.4 API)
//! <https://www.gdac.com/>

use super::{midpoint, AskBook, BidBook};
use crate::{
    config::HttpsConfig,
    https_client::{HttpsClient, Query},
    prelude::*,
};
use crate::{Error, Price, PriceQuantity, TradingPair};
use serde::{de, Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

/// Base URI for requests to the GDAC v0.4 API
pub const API_HOST: &str = "partner.gdac.com";

/// Source provider for GDAC
pub struct GdacSource {
    https_client: HttpsClient,
}

impl GdacSource {
    /// Create a new GDAC source provider
    pub fn new(config: &HttpsConfig) -> Result<Self, Error> {
        let https_client = HttpsClient::new(API_HOST, config)?;
        Ok(Self { https_client })
    }

    /// Get trading pairs
    pub async fn trading_pairs(&self, pair: &TradingPair) -> Result<Price, Error> {
        info!("Getting DGAC Trading Pair {}", pair);

        let mut query = Query::new();
        query.add("pair".to_owned(), pair.percent_encode());

        let api_response: Quote = self
            .https_client
            .get_json("/v0.4/public/orderbook", &query)
            .await?;
        info!("Got DGAC Trading Pair {}", pair);

        midpoint(&api_response)
    }
}

/// Quoted prices as sourced from the order book
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quote {
    /// Ask price
    pub ask: Vec<PricePoint>,

    /// Bid price
    pub bid: Vec<PricePoint>,
}

///This trait returns a vector of ask prices and quantities
impl AskBook for Quote {
    fn asks(&self) -> Result<Vec<PriceQuantity>, Error> {
        self.ask
            .iter()
            .map(|p| {
                p.volume
                    .parse()
                    .map(|quantity| PriceQuantity {
                        price: p.price,
                        quantity,
                    })
                    .map_err(Into::into)
            })
            .collect()
    }
}

///This trait returns a vector of bid prices and quantities
impl BidBook for Quote {
    fn bids(&self) -> Result<Vec<PriceQuantity>, Error> {
        self.bid
            .iter()
            .map(|p| {
                p.volume
                    .parse()
                    .map(|quantity| PriceQuantity {
                        price: p.price,
                        quantity,
                    })
                    .map_err(Into::into)
            })
            .collect()
    }
}

/// Prices and associated volumes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PricePoint {
    /// Price
    pub price: Price,

    /// Volume
    pub volume: String,
}

/// Error responses
#[derive(Clone, Debug, Deserialize)]
pub struct ErrorResponse {
    /// Response code
    pub code: ErrorCode,

    /// Response data
    pub data: serde_json::Value,
}

/// Error response codes
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    /// Application-level error
    InternalError,

    /// Not found (404) or bad HTTP method
    Unavailable,

    /// Unrecognized error codes
    Other(String),
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ErrorCode::InternalError => "__internal_error__",
            ErrorCode::Unavailable => "__unavailable__",
            ErrorCode::Other(other) => other.as_ref(),
        })
    }
}

impl FromStr for ErrorCode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(match s {
            "__internal_error__" => ErrorCode::InternalError,
            "__unavailable__" => ErrorCode::Unavailable,
            other => ErrorCode::Other(other.to_owned()),
        })
    }
}

impl std::error::Error for ErrorCode {}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use de::Error;
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::GdacSource;
    use std::future::Future;

    fn block_on<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future)
    }

    /// `trading_pairs()` test with known currency pair
    #[test]
    #[ignore]
    fn trading_pairs_ok() {
        let pair = "LUNA/KRW".parse().unwrap();
        let _price = block_on(
            GdacSource::new(&Default::default())
                .unwrap()
                .trading_pairs(&pair),
        )
        .unwrap();
    }

    /// `trading_pairs()` with invalid currency pair
    #[test]
    #[ignore]
    fn trading_pairs_404() {
        let pair = "N/A".parse().unwrap();

        let _err = block_on(
            GdacSource::new(&Default::default())
                .unwrap()
                .trading_pairs(&pair),
        )
        .err()
        .unwrap();
    }
}
