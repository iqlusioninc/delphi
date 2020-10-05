//! Dunamu Source Provider (v0.4 API)
//! <https://www.dunamu.com/>

use crate::{
    error::{Error, ErrorKind},
    prelude::*,
};
use crate::{Currency, TradingPair};
use bytes::buf::ext::BufExt;
use hyper::{
    client::{Client, HttpConnector},
    header, Body, Request,
};
use hyper_rustls::HttpsConnector;
use serde::{Deserialize, Serialize};

//https://quotation-api-cdn.dunamu.com/v1/forex/recent?codes=FRX.KRWUSD

/// Base URI for requests to the Dunamu API
pub const BASE_URI: &str = "https://quotation-api-cdn.dunamu.com";

/// User-Agent to send in HTTP request
pub const USER_AGENT: &str = "iqlusion delphi";

/// Source provider for Dunamu
pub struct DunamuSource {
    http_client: Client<HttpsConnector<HttpConnector>>,
}

impl DunamuSource {
    /// Create a new Dunamu source provider
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            http_client: Client::builder().build(HttpsConnector::new()),
        }
    }

    /// Get trading pairs
    pub async fn trading_pairs(&self, pair: &TradingPair) -> Result<Response, Error> {
        if pair.0 != Currency::Krw && pair.1 != Currency::Krw {
            fail!(ErrorKind::Currency, "trading pair must be with KRW");
        }

        let uri = format!(
            "{}/v1/forex/recent?codes=FRX.{}{}",
            BASE_URI, pair.0, pair.1
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
        let body = hyper::body::aggregate(response.into_body()).await?;
        Ok(serde_json::from_reader(body.reader())?)
    }
}

/// API responses Vector
pub type Response = Vec<ResponseElement>;
/// API response entity
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseElement {
    code: String,
    currency_code: String,
    currency_name: String,
    country: String,
    name: String,
    date: String,
    time: String,
    recurrence_count: i64,
    base_price: f64,
    opening_price: f64,
    high_price: f64,
    low_price: f64,
    change: String,
    change_price: f64,
    cash_buying_price: f64,
    cash_selling_price: f64,
    tt_buying_price: f64,
    tt_selling_price: f64,
    tc_buying_price: Option<serde_json::Value>,
    fc_selling_price: Option<serde_json::Value>,
    exchange_commission: f64,
    us_dollar_rate: f64,
    #[serde(rename = "high52wPrice")]
    high52_w_price: f64,
    #[serde(rename = "high52wDate")]
    high52_w_date: String,
    #[serde(rename = "low52wPrice")]
    low52_w_price: f64,
    #[serde(rename = "low52wDate")]
    low52_w_date: String,
    currency_unit: i64,
    provider: String,
    timestamp: i64,
    id: i64,
    created_at: String,
    modified_at: String,
    change_rate: f64,
    signed_change_price: f64,
    signed_change_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::DunamuSource;
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
        let pair = "KRW/USD".parse().unwrap();
        let response = block_on(DunamuSource::new().trading_pairs(&pair)).unwrap();

        assert!(response.len() > 0);
    }

    /// `trading_pairs()` with invalid currency pair
    #[test]
    #[ignore]
    fn trading_pairs_404() {
        let pair = "N/A".parse().unwrap();

        // TODO(tarcieri): test 404 handling
        let _err = block_on(DunamuSource::new().trading_pairs(&pair))
            .err()
            .unwrap();
    }
}