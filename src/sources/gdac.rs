//! GDAC Source Provider (v0.4 API)
//! <https://www.gdac.com/>

use super::{AskBook, BidBook, Pair, Price, PriceQuantity};
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
use serde::{de, Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

/// Base URI for requests to the GDAC v0.4 API
pub const BASE_URI_V4: &str = "https://partner.gdac.com/v0.4";

/// User-Agent to send in HTTP request
pub const USER_AGENT: &str = "iqlusion delphi";

/// Source provider for GDAC
pub struct GdacSource {
    http_client: Client<HttpsConnector<HttpConnector>>,
}

impl GdacSource {
    /// Create a new GDAC source provider
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            http_client: Client::builder().build(HttpsConnector::new()),
        }
    }

    /// Get trading pairs
    pub async fn trading_pairs(&self, pair: &Pair) -> Result<Quote, Error> {
        let uri = format!(
            "{}/public/orderbook?pair={}",
            BASE_URI_V4,
            pair.percent_encode()
        );

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
                        "got {} error status with malformed body: {}",
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
                        price: p.price.clone(),
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
                        price: p.price.clone(),
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
    use super::{ErrorCode, GdacSource};
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
        let quote = block_on(GdacSource::new().trading_pairs(&pair)).unwrap();
        dbg!(&quote);
        assert!(quote.ask.len() > 10);
        assert!(quote.bid.len() > 10);
    }

    /// `trading_pairs()` with invalid currency pair
    #[test]
    #[ignore]
    fn trading_pairs_404() {
        let pair = "N/A".parse().unwrap();
        let quote_err = block_on(GdacSource::new().trading_pairs(&pair))
            .err()
            .unwrap();

        use std::error::Error;
        let err: &ErrorCode = quote_err.source().unwrap().downcast_ref().unwrap();

        assert_eq!(err, &ErrorCode::InternalError);
    }
}
