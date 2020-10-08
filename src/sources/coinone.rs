//! Coinone Source Provider
//! <https://coinone.co.kr/>
//!
//! Only KRW pairs are supported.

use super::{midpoint, AskBook, BidBook};
use crate::{
    config::HttpsConfig,
    https_client::{HttpsClient, Query},
};
use crate::{prelude::*, Currency, Error, ErrorKind, Price, PriceQuantity, TradingPair};
use serde::{Deserialize, Serialize};

/// Hostname for Coinone API
pub const API_HOST: &str = "api.coinone.co.kr";

/// Source provider for Coinone
pub struct CoinoneSource {
    https_client: HttpsClient,
}

impl CoinoneSource {
    /// Create a new Coinone source provider
    pub fn new(config: &HttpsConfig) -> Result<Self, Error> {
        let https_client = HttpsClient::new(API_HOST, config)?;
        Ok(Self { https_client })
    }

    /// Get trading pairs
    pub async fn trading_pairs(&self, pair: &TradingPair) -> Result<Price, Error> {
        if pair.1 != Currency::Krw {
            fail!(ErrorKind::Currency, "trading pair must be with KRW");
        }

        let mut query = Query::new();
        query.add("currency".to_owned(), pair.0.to_string());

        let api_response: Response = self.https_client.get_json("/orderbook", &query).await?;
        midpoint(&api_response)
    }
}

/// API responses
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    /// Error code
    #[serde(rename = "errorCode")]
    pub error_code: String,

    /// Result of the operation
    pub result: String,

    /// Requested currency
    pub currency: String,

    /// Timestamp
    pub timestamp: String,

    /// Ask prices
    pub ask: Vec<PricePoint>,

    /// Bid prices
    pub bid: Vec<PricePoint>,
}

///This trait returns a vector of ask prices and quantities
impl AskBook for Response {
    fn asks(&self) -> Result<Vec<PriceQuantity>, Error> {
        self.ask
            .iter()
            .map(|p| {
                p.qty
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
impl BidBook for Response {
    fn bids(&self) -> Result<Vec<PriceQuantity>, Error> {
        self.bid
            .iter()
            .map(|p| {
                p.qty
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

    /// Quantity
    pub qty: String,
}

#[cfg(test)]
mod tests {
    use super::CoinoneSource;
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
            CoinoneSource::new(&Default::default())
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

        // TODO(tarcieri): test 404 handling
        let _err = block_on(
            CoinoneSource::new(&Default::default())
                .unwrap()
                .trading_pairs(&pair),
        )
        .err()
        .unwrap();
    }
}
