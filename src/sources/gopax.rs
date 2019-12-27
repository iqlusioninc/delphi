//! GOPAX Source Provider
//! <https://www.gopax.com/API/>
//! <https://api.gopax.co.kr/trading-pairs/LUNA-KRW/book>

use super::Pair;
use crate::error::Error;
use bytes::buf::ext::BufExt;
use hyper::{
    client::{Client, HttpConnector},
    header, Body, Request,
};
use hyper_rustls::HttpsConnector;
use serde::{Deserialize, Serialize};

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
            panic!("got error response {:?}", status)
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

    //    //// `trading_pairs() with invalid currency pair
    //    // #[test]
    //    #[ignore]
    //    fn trading_pairs_404() {
    //        let pair = "N/A".parse().unwrap();
    //        let quote = block_on(GopaxSource::new().trading_pairs(&pair))
    //            .unwrap();
    //
    //        dbg!(&quote);
    //    }
}
