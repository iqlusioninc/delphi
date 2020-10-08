//! GOPAX Source Provider
//! <https://www.gopax.co.id/API/>
//! <https://api.gopax.co.kr/trading-pairs/LUNA-KRW/book>

use super::{midpoint, AskBook, BidBook};
use crate::{
    config::HttpsConfig,
    https_client::{HttpsClient, Query},
    Error, Price, PriceQuantity, TradingPair,
};
use rust_decimal::Decimal;
use serde::{de, Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Base URI for requests to the GOPAX API
pub const API_HOST: &str = "api.gopax.co.kr";

/// Source provider for GOPAX
pub struct GopaxSource {
    https_client: HttpsClient,
}

impl GopaxSource {
    /// Create a new GOPAX source provider
    #[allow(clippy::new_without_default)]
    pub fn new(config: &HttpsConfig) -> Result<Self, Error> {
        let https_client = HttpsClient::new(API_HOST, config)?;
        Ok(Self { https_client })
    }

    /// Get trading pairs
    pub async fn trading_pairs(&self, pair: &TradingPair) -> Result<Price, Error> {
        let query = Query::new();

        let api_response: Response = self
            .https_client
            .get_json(
                &format!("/trading-pairs/{}-{}/book", pair.0, pair.1),
                &query,
            )
            .await?;

        midpoint(&api_response)
    }
}

/// Quoted prices as sourced from the order book
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    /// Sequence
    pub sequence: u64,

    /// Bid price
    pub bid: Vec<PricePoint>,

    /// Ask price
    pub ask: Vec<PricePoint>,
}

///This trait returns a vector of ask prices and quantities
impl AskBook for Response {
    fn asks(&self) -> Result<Vec<PriceQuantity>, Error> {
        Ok(self
            .ask
            .iter()
            .map(|p| PriceQuantity {
                price: Price::new(p.price).unwrap(),
                quantity: p.volume,
            })
            .collect())
    }
}

///This trait returns a vector of bid prices and quantities
impl BidBook for Response {
    fn bids(&self) -> Result<Vec<PriceQuantity>, Error> {
        Ok(self
            .bid
            .iter()
            .map(|p| PriceQuantity {
                price: Price::new(p.price).unwrap(),
                quantity: p.volume,
            })
            .collect())
    }
}
/// Prices and associated volumes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PricePoint {
    /// Id
    pub id: String,

    /// Price
    #[serde(deserialize_with = "deserialize_decimal")]
    pub price: Decimal,

    /// Volume
    #[serde(deserialize_with = "deserialize_decimal")]
    pub volume: Decimal,

    /// Timestamp
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub timestamp: SystemTime,
}

/// Deserialize decimal value
fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: de::Deserializer<'de>,
{
    // TODO: avoid floating point/string conversions
    let value = f64::deserialize(deserializer)?;
    value.to_string().parse().map_err(de::Error::custom)
}

/// Deserialize timestamp value
fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|s| s.parse().map_err(de::Error::custom))
        .map(|unix_secs| UNIX_EPOCH + Duration::from_secs(unix_secs))
}

#[cfg(test)]
mod tests {
    use super::GopaxSource;
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
        let _quote = block_on(
            GopaxSource::new(&Default::default())
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
        let quote = block_on(
            GopaxSource::new(&Default::default())
                .unwrap()
                .trading_pairs(&pair),
        );
        assert!(quote.is_err());
    }
}
