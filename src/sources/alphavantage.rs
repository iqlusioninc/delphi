//! Alphavantage Source Provider
//! <https://www.alphavantage.co/>
//!

use crate::{config::HttpsConfig, prelude::*, Error, ErrorKind, Price, TradingPair};
use iqhttp::{HttpsClient, Query};
use serde::{Deserialize, Serialize};

/// Hostname for AlphaVantage API
pub const API_HOST: &str = "www.alphavantage.co";

/// Source provider for AlphaVantage
pub struct AlphavantageSource {
    https_client: HttpsClient,
    apikey: String,
}

///Parameters for queries
pub struct AlphavantageParams {
    function: String,
    from_currency: String,
    to_currency: String,
    apikey: String,
}

impl AlphavantageParams {
    ///Convert params into url query parameters
    pub fn to_request_uri(&self) -> Query {
        let mut query = Query::new();
        query.add("function".to_owned(), self.function.to_string());
        query.add("from_currency".to_owned(), self.from_currency.to_string());
        query.add("to_currency".to_owned(), self.to_currency.to_string());
        query.add("apikey".to_owned(), self.apikey.to_string());

        query
    }
}

impl AlphavantageSource {
    /// Create a new Alphavantage source provider
    pub fn new(apikey: impl Into<String>, config: &HttpsConfig) -> Result<Self, Error> {
        let https_client = config.new_client(API_HOST)?;
        Ok(Self {
            https_client,
            apikey: apikey.into(),
        })
    }

    /// Get trading pairs
    pub async fn trading_pairs(&self, pair: &TradingPair) -> Result<Price, Error> {
        let params = AlphavantageParams {
            function: "CURRENCY_EXCHANGE_RATE".to_owned(),
            from_currency: pair.0.to_string(),
            to_currency: pair.1.to_string(),
            apikey: self.apikey.clone(),
        };

        let query = params.to_request_uri();
        match self.https_client.get_json("/query", &query).await? {
            Response::Success(resp) => Ok(resp.exchange_rate),
            Response::Error(msg) => fail!(ErrorKind::Source, "Alpha Vantage error: {}", msg),
        }
    }
}

/// Outer struct of the API responses
#[derive(Serialize, Deserialize)]
pub enum Response {
    /// Successful API response
    #[serde(rename = "Realtime Currency Exchange Rate")]
    Success(RealtimeCurrencyExchangeRate),

    /// Error response
    #[serde(rename = "Note")]
    Error(String),
}

#[derive(Serialize, Deserialize)]
///Inner struct of the API response
pub struct RealtimeCurrencyExchangeRate {
    #[serde(rename = "1. From_Currency Code")]
    from_currency_code: String,
    #[serde(rename = "2. From_Currency Name")]
    from_currency_name: String,
    #[serde(rename = "3. To_Currency Code")]
    to_currency_code: String,
    #[serde(rename = "4. To_Currency Name")]
    to_currency_name: String,
    #[serde(rename = "5. Exchange Rate")]
    /// Quote of the exchange rate Price
    pub exchange_rate: Price,
    #[serde(rename = "6. Last Refreshed")]
    timestamp: String,
    #[serde(rename = "7. Time Zone")]
    timezone: String,
    #[serde(rename = "8. Bid Price")]
    ///Quote for bid price
    pub bid: Price,
    #[serde(rename = "9. Ask Price")]
    ///Quote for ask price
    pub ask: Price,
}

#[cfg(test)]
mod tests {
    use super::AlphavantageSource;
    #[tokio::test]
    #[ignore]
    async fn trading_pairs_ok() {
        let pair = "KRW/USD".parse().unwrap();
        let _response = AlphavantageSource::new(
            &std::env::var("ALPHAVANTAGE_API")
                .expect("Please set the ALPHAVANTAGE_API env variable"),
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
    //     let _err = block_on(AlphavantageSource::new().trading_pairs(&pair))
    //         .err()
    //         .unwrap();
    // }
}
