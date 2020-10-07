//! Alphavantage Source Provider
//! <https://www.alphavantage.co/>
//!

use super::USER_AGENT;
use crate::{Error, Price, TradingPair};
use bytes::buf::ext::BufExt;
use hyper::{
    client::{Client, HttpConnector},
    header, Body, Request, Uri,
};
use hyper_rustls::HttpsConnector;
use serde::{Deserialize, Serialize};

/// Base URI for requests to the Coinone API
pub const BASE_URI: &str = "https://www.alphavantage.co/";

/// Source provider for Alphavantage
pub struct AlphavantageSource {
    http_client: Client<HttpsConnector<HttpConnector>>,
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
    pub fn to_request_uri(&self) -> Uri {
        let query = [
            ("function", &self.function),
            ("from_currency", &self.from_currency),
            ("to_currency", &self.to_currency),
            ("apikey", &self.apikey),
        ]
        .iter()
        .map(|(k, v)| format!("{}={}", k, v)) // TODO: add urlencoding
        .collect::<Vec<_>>()
        .join("&");

        let path_and_query = format!("/query?{}", query);

        Uri::builder()
            .scheme("https")
            .authority("www.alphavantage.co")
            .path_and_query(path_and_query.as_str())
            .build()
            .unwrap()
    }
}

impl AlphavantageSource {
    /// Create a new Alphavantage source provider
    pub fn new(apikey: impl Into<String>) -> Self {
        Self {
            http_client: Client::builder().build(HttpsConnector::new()),
            apikey: apikey.into(),
        }
    }

    /// Get trading pairs
    pub async fn trading_pairs(&self, pair: &TradingPair) -> Result<Response, Error> {
        let params = AlphavantageParams {
            function: "CURRENCY_EXCHANGE_RATE".to_owned(),
            from_currency: pair.0.to_string(),
            to_currency: pair.1.to_string(),
            apikey: self.apikey.clone(),
        };

        let uri = params.to_request_uri();

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

/// Outer struct of the API responses
#[derive(Serialize, Deserialize)]
pub struct Response {
    #[serde(rename = "Realtime Currency Exchange Rate")]
    ///API response
    pub realtime_currency_exchange_rate: RealtimeCurrencyExchangeRate,
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
        )
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
