//! Bithumb Source Provider
//! <https://api.bithumb.com/public/ticker/luna_krw>
//!
//! Only KRW pairs are supported.

use crate::{
    config::HttpsConfig,
    https_client::{HttpsClient, Query},
};
use crate::{prelude::*, Currency, Error, ErrorKind, Price, TradingPair};
use serde::{Deserialize, Serialize};

/// Hostname for Bithumb API
pub const API_HOST: &str = "api.bithumb.com";

/// Source provider for Bithumb
pub struct BithumbSource {
    https_client: HttpsClient,
}

impl BithumbSource {
    /// Create a new Bithumb source provider
    pub fn new(config: &HttpsConfig) -> Result<Self, Error> {
        let https_client = HttpsClient::new(API_HOST, config)?;
        Ok(Self { https_client })
    }

    /// Get trading pairs
    pub async fn trading_pairs(&self, pair: &TradingPair) -> Result<Price, Error> {
        if pair.1 != Currency::Krw {
            fail!(ErrorKind::Currency, "trading pair must be with KRW");
        }

        let query = Query::new();

        let api_response: Response = self
            .https_client
            .get_json("/public/ticker/luna_krw", &query)
            .await?;
        Ok(api_response.data.closing_price.parse::<Price>().unwrap())
    }
}

/// API responses
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    status: String,
    ///data response
    pub data: Data,
}

///Data response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Data {
    opening_price: String,
    ///closing price response
    pub closing_price: String,
    min_price: String,
    max_price: String,
    units_traded: String,
    acc_trade_value: String,
    prev_closing_price: String,
    #[serde(rename = "units_traded_24H")]
    units_traded_24_h: String,
    #[serde(rename = "acc_trade_value_24H")]
    acc_trade_value_24_h: String,
    #[serde(rename = "fluctate_24H")]
    fluctate_24_h: String,
    #[serde(rename = "fluctate_rate_24H")]
    fluctate_rate_24_h: String,
    date: String,
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
    use super::BithumbSource;
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
            BithumbSource::new(&Default::default())
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
            BithumbSource::new(&Default::default())
                .unwrap()
                .trading_pairs(&pair),
        )
        .err()
        .unwrap();
    }
}
