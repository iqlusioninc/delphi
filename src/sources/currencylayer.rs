//! Currencylayer Source Provider
//! <https://api.currencylayer.com>
//!

use crate::{
    config::HttpsConfig,
    https_client::{HttpsClient, Query},
    prelude::*,
};
use crate::{Error, Price, TradingPair};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
/// Hostname for Currencylayer API
pub const API_HOST: &str = "api.currencylayer.com";

/// Source provider for Currencylayer
pub struct CurrencylayerSource {
    https_client: HttpsClient,
    access_key: String,
}

///Parameters for queries
pub struct CurrencylayerParams {
    source: String,
    currencies: String,
    access_key: String,
}

impl CurrencylayerParams {
    ///Convert params into url query parameters
    pub fn to_request_uri(&self) -> Query {
        let mut query = Query::new();
        query.add("source".to_owned(), self.source.to_string());
        query.add("currencies".to_owned(), self.currencies.to_string());
        query.add("access_key".to_owned(), self.access_key.to_string());

        query
    }
}

impl CurrencylayerSource {
    /// Create a new Currencylayer source provider
    pub fn new(access_key: impl Into<String>, config: &HttpsConfig) -> Result<Self, Error> {
        let https_client = HttpsClient::new(API_HOST, config)?;
        Ok(Self {
            https_client,
            access_key: access_key.into(),
        })
    }

    /// Get trading pairs
    pub async fn trading_pairs(&self, pair: &TradingPair) -> Result<Price, Error> {
        let params = CurrencylayerParams {
            source: pair.0.to_string(),
            currencies: pair.1.to_string(),
            access_key: self.access_key.clone(),
        };

        let query = params.to_request_uri();
        let resp: Response = self.https_client.get_json("/live", &query).await?;
        let price = resp
            .quotes
            .values()
            .into_iter()
            .last()
            .expect("expected currencylayer to return one value");
        let dec_price = Decimal::from_f64(*price)
            .expect("expected currencylayer response to convert to a decimal");
        info!("Got CurrencyLayer Trading Pair {}", pair);

        Price::new(dec_price)
    }
}

/// Outer struct of the API responses
#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    success: bool,
    terms: String,
    privacy: String,
    timestamp: i64,
    source: String,
    quotes: std::collections::HashMap<String, f64>,
}

#[cfg(test)]
mod tests {
    use super::CurrencylayerSource;
    #[tokio::test]
    #[ignore]
    async fn trading_pairs_ok() {
        let pair = "KRW/USD".parse().unwrap();
        let _response = CurrencylayerSource::new(
            &std::env::var("CURRENCYLAYER_API")
                .expect("Please set the CURRENCYLAYER_API env variable"),
            &Default::default(),
        )
        .unwrap()
        .trading_pairs(&pair)
        .await
        .unwrap();
    }

    // / `trading_pairs()` with invalid currency pair
    // #[test]
    // #[ignore]
    // fn trading_pairs_404() {
    //     let pair = "N/A".parse().unwrap();

    //     // TODO(tarcieri): test 404 handling
    //     let _err = block_on(CurrencylayerSource::new().trading_pairs(&pair))
    //         .err()
    //         .unwrap();
    // }
}
