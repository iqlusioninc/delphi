//! IMF SDR Source Provider
//! <https://www.imf.org/>

use super::{Currency, Pair, Price};
use crate::{
    error::{Error, ErrorKind},
    prelude::*,
};
use bytes::buf::ext::BufExt;
use csv;
use hyper::{
    client::{Client, HttpConnector},
    header, Body, Request,
};
use hyper_rustls::HttpsConnector;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

//https://www.imf.org/external/np/fin/data/rms_five.aspx?tsvflag=Y"

/// Base URI for requests to the Coinone API
pub const BASE_URI: &str = "https://www.imf.org";

/// User-Agent to send in HTTP request
pub const USER_AGENT: &str = "iqlusion delphi";

/// The IMF returns tab seperated data is form that is designed to for

pub struct ImfSDRSource {
    http_client: Client<HttpsConnector<HttpConnector>>,
}
/// importing into Excel spreadsheets rather than being machine
/// friendly.record. The TSV data has irregular structure.
/// The strategy here is to find the subsection of data we are
/// looking for by trying to deserialize each row into IMFSDRRow and
/// ignoring deserialization errors. The IMF provides data for the
/// last 5 days but data may no be available for all days.

#[derive(Debug, Deserialize)]
struct IMFSDRRow {
    currency: String,
    price_0: Option<Price>,
    price_1: Option<Price>,
    price_2: Option<Price>,
    price_3: Option<Price>,
    price_4: Option<Price>,
}

impl IMFSDRRow {
    /// Best price is the most recent price. Use
    fn response_from_best_price(&self) -> Option<Response> {
        if let Some(ref price) = self.price_4 {
            return Some(Response {
                price: price.clone(),
            });
        }
        if let Some(ref price) = self.price_3 {
            return Some(Response {
                price: price.clone(),
            });
        }
        if let Some(ref price) = self.price_2 {
            return Some(Response {
                price: price.clone(),
            });
        }

        if let Some(ref price) = self.price_1 {
            return Some(Response {
                price: price.clone(),
            });
        }
        if let Some(ref price) = self.price_0 {
            return Some(Response {
                price: price.clone(),
            });
        }

        return None;
    }
}

impl ImfSDRSource {
    /// Create a new Dunamu source provider
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
        if pair.1 != Currency::SDR {
            fail!(ErrorKind::Currency, "trading pair must be with IMF SDR");
        }

        let uri = format!("{}/external/np/fin/data/rms_five.aspx?tsvflag=Y", BASE_URI,);

        let mut request = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(Body::empty())?;

        {
            let headers = request.headers_mut();
            headers.insert(
                header::USER_AGENT,
                format!("{}/{}", USER_AGENT, env!("CARGO_PKG_VERSION"))
                    .parse()
                    .unwrap(),
            );
        }

        let response = self.http_client.request(request).await?;
        let body = hyper::body::aggregate(response.into_body()).await?;

        let mut imf_sdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .delimiter(b'\t')
            .from_reader(body.reader());

        let mut response_row: Option<IMFSDRRow> = None;

        for result in imf_sdr.records() {
            let record = result.map_err(|e| {
                format_err!(ErrorKind::Source, "got error with malformed csv: {}", e)
            })?;

            let row: Result<IMFSDRRow, csv::Error> = record.deserialize(None);

            match row {
                Ok(imf_sdr_row) => {
                    if imf_sdr_row.currency == pair.0.imf_long_name() {
                        response_row = Some(imf_sdr_row);
                        break;
                    }
                }
                Err(_e) => continue,
            };
        }

        match response_row {
            Some(resp) => Response::try_from(resp)
                .map_err(|e| format_err!(ErrorKind::Parse, "{}, {}", e, pair).into()),
            None => Err(format_err!(ErrorKind::Parse, "price for {} not found", pair).into()),
        }
    }
}
/// Provides a single price point for a currency pair based extracted data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    price: Price,
}

impl TryFrom<IMFSDRRow> for Response {
    type Error = &'static str;
    fn try_from(row: IMFSDRRow) -> Result<Self, Self::Error> {
        match row.response_from_best_price() {
            Some(resp) => Ok(resp),
            None => Err("No price data found for for currency pair"),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::ImfSDRSource;
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
        let pair = "KRW/SDR".parse().unwrap();
        let quote = block_on(ImfSDRSource::new().trading_pairs(&pair)).unwrap();
        dbg!(&quote);
    }

    // `trading_pairs()` with invalid currency pair
    // #[test]
    // #[ignore]
    // fn trading_pairs_404() {
    //     let pair = "N/A".parse().unwrap();
    //     let quote_err = block_on(ImfSDRSource::new().trading_pairs(&pair))
    //         .err()
    //         .unwrap();

    //    use std::error::Error;
    //    let err: &ErrorCode = quote_err.source().unwrap().downcast_ref().unwrap();

    //     assert_eq!(err, &ErrorCode::InternalError);
    // }
}
