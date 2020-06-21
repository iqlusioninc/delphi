//! Coinone Source Provider
//! <https://coinone.co.kr/>
//!
//! Only KRW pairs are supported.

use super::{Currency, Pair, Price, ComputablePrice};
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
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use rust_decimal::{prelude::*, Decimal};



use async_trait::async_trait;


/// Base URI for requests to the Coinone API
pub const BASE_URI: &str = "https://api.coinone.co.kr/";

/// User-Agent to send in HTTP request
pub const USER_AGENT: &str = "iqlusion delphi";

/// Source provider for Coinone
pub struct CoinoneSource {
    http_client: Client<HttpsConnector<HttpConnector>>,
}

impl CoinoneSource {
    /// Create a new Coinone source provider
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            http_client: Client::builder()
                .keep_alive(true)
                .build(HttpsConnector::new()),
        }
    }

    /// Get trading pairs
    pub async fn trading_pairs(&self, pair: &Pair) -> Result<Response, Error> {
        if pair.1 != Currency::Krw {
            fail!(ErrorKind::Currency, "trading pair must be with KRW");
        }

        let uri = format!("{}/orderbook?currency={}", BASE_URI, pair.0);

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

#[async_trait]
impl ComputablePrice for CoinoneSource{

    async fn run_price(&self, pair:Pair)-> Result<Price,Error>{
        let resp = self.trading_pairs(&pair).await?;
        //Average the weighted averages between bid and ask
        Price::new((resp.ask_weighted_average()?.0 + resp.bid_weighted_average()?.0)/Decimal::new(2, 0))

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

impl Response {
    fn ask_weighted_average(&self)->Result<Price, Error> {

        let mut price_sum_product = Decimal::zero();
        let mut total = Decimal::zero();
        for ask_pp in &self.ask {
            let quantity = Decimal::from_str(&ask_pp.qty.clone())?;
            price_sum_product += ask_pp.price.0 * quantity;
            total += quantity;
        }
        Price::new(price_sum_product / total)

    }

    fn bid_weighted_average(&self)->Result<Price, Error>{

        let mut price_sum_product = Decimal::zero();
        let mut total = Decimal::zero();
        for bid_pp in &self.bid {
            let quantity = Decimal::from_str(&bid_pp.qty)?;
            price_sum_product += bid_pp.price.0 * quantity;
            total += quantity;
        }
        Price::new(price_sum_product / total)
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
        let response = block_on(CoinoneSource::new().trading_pairs(&pair)).unwrap();
        assert!(response.ask.len() > 10);
        assert!(response.bid.len() > 10);
    }

    /// `trading_pairs()` with invalid currency pair
    #[test]
    #[ignore]
    fn trading_pairs_404() {
        let pair = "N/A".parse().unwrap();

        // TODO(tarcieri): test 404 handling
        let _err = block_on(CoinoneSource::new().trading_pairs(&pair))
            .err()
            .unwrap();
    }
}
