//! GOPAX Source Provider
//! <https://www.gopax.com/API/>
//! <https://api.gopax.co.kr/trading-pairs/LUNA-KRW/book>

use super::Pair;
use crate::{
    error::{Error, ErrorKind},
    prelude::*,
};
use abscissa_core::error::message::Message;
use bytes::buf::ext::BufExt;
use hyper::{
    client::{Client, HttpConnector},
    header, Body, Request,
};
use hyper_rustls::HttpsConnector;
use serde::{de, Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};
use thiserror::Error;

/// Base URI for requests to the GOPAX API
pub const BASE_URI_V4: &str = "https://api.gopax.co.kr";

/// User-Agent to send in HTTP request
pub const USER_AGENT: &str = "iqlusion delphi";

/// Source provider for GDAC
pub struct GopaxSource {
    http_client: Client<HttpsConnector<HttpConnector>>,
}

impl GopaxSource {
    /// Create a new GOPAX source provider
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            http_client: Client::builder()
                .keep_alive(true)
                .build(HttpsConnector::new()),
        }
    }

    /// Get trading pairs
    pub async fn trading_pairs(&self, pair: &Pair) -> Result<Quote, Error> {
        let uri = format!("{}/trading-pairs/{}-{}/book", BASE_URI_V4, pair.0, pair.1);
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
            let error_response: ErrorResponse =
                serde_json::from_reader(body.reader()).map_err(|e| {
                    format_err!(
                        ErrorKind::Source,
                        "got {:?} error status with malformed body: {}",
                        status,
                        e
                    )
                })?;

            let err = ErrorKind::Source.context(error_response.code);
            return Err(err.into());
        }

        Ok(serde_json::from_reader(body.reader())?)
    }
}

/// Quoted prices as sourced from the order book
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quote {
    /// Sequence
    pub sequence: u64,

    /// Bid price
    pub bid: Vec<PricePoint>,

    /// Ask price
    pub ask: Vec<PricePoint>,
}

/// Prices and associated volumes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PricePoint(String, f64, f64);

/// Error response codes
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ErrorCode {
    /// Returns 200 but semantic logical error
    #[error("SemanticError encountered")]
    SemanticError(Message),

    #[error("HTTPError encountered")]
    /// HTTP status codes
    HttpError(u16),
}

impl std::error::Error for ErrorCode {}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ErrorCode::SemanticError(s) => s.as_ref(),
            ErrorCode::HttpError(u) => u,
        })
    }
}

impl FromStr for ErrorCode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(match s {
            other => ErrorCode::SemanticError(other.to_owned()),
        })
    }
}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use de::Error;
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(D::Error::custom)
    }
}

/// Error responses
#[derive(Clone, Debug, Deserialize)]
pub struct ErrorResponse {
    /// Response code
    pub code: ErrorCode,

    /// Response data
    pub data: serde_json::Value,
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
        dbg!(&quote);
        assert!(quote.ask.len() > 10);
        assert!(quote.bid.len() > 10);
    }

    /// `trading_pairs() with invalid currency pair
    #[test]
    #[ignore]
    fn trading_pairs_404() {
        let pair = "N/A".parse().unwrap();
        let quote = block_on(GopaxSource::new().trading_pairs(&pair)).unwrap();
        dbg!(&quote);
    }
}
