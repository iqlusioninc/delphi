//! GOPAX Source Provider
//! <https://www.gopax.co.id/API/>
//! <https://api.gopax.co.kr/trading-pairs/LUNA-KRW/book>

use super::{AskBook, BidBook, Pair, Price, PriceQuantity, USER_AGENT};
use crate::{
    error::{Error, ErrorKind},
    prelude::*,
};
use bytes::buf::ext::BufExt;
use hyper::{
    client::{Client, HttpConnector},
    header, Body, Request,
};
use hyper_rustls::HttpsConnector;
use rust_decimal::Decimal;
use serde::{de, Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Base URI for requests to the GOPAX API
pub const BASE_URI: &str = "https://api.gopax.co.kr";

/// Source provider for GOPAX
pub struct GopaxSource {
    http_client: Client<HttpsConnector<HttpConnector>>,
}

impl GopaxSource {
    /// Create a new GOPAX source provider
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            http_client: Client::builder().build(HttpsConnector::new()),
        }
    }

    /// Get trading pairs
    pub async fn trading_pairs(&self, pair: &Pair) -> Result<Response, Error> {
        let uri = format!("{}/trading-pairs/{}-{}/book", BASE_URI, pair.0, pair.1);
        dbg!(&uri);

        let mut request = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(Body::empty())?;

        {
            let headers = request.headers_mut();
            headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
            headers.insert(
                header::USER_AGENT,
                format!("{}/{}", USER_AGENT, env!("CARGO_PKG_VERSION"))
                    .parse()
                    .unwrap(),
            );
        }

        let response = self.http_client.request(request).await?;
        let status = response.status();
        let body = hyper::body::aggregate(response.into_body()).await?;

        if !status.is_success() {
            fail!(ErrorKind::Source, "got {:?} error status", status)
        }

        serde_json::from_reader(body.reader()).map_err(|e| {
            format_err!(
                ErrorKind::Source,
                "couldn't parse api.gopax.co.kr response: {}",
                e
            )
            .into()
        })
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
        let quote = block_on(GopaxSource::new().trading_pairs(&pair)).unwrap();
        assert!(quote.ask.len() > 10);
        assert!(quote.bid.len() > 10);
    }

    /// `trading_pairs()` with invalid currency pair
    #[test]
    #[ignore]
    fn trading_pairs_404() {
        let pair = "N/A".parse().unwrap();
        let quote = block_on(GopaxSource::new().trading_pairs(&pair));
        assert!(quote.is_err());
    }
}
