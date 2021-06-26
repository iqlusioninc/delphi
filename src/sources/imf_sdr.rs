//! IMF SDR Source Provider
//! <https://www.imf.org/>

use crate::{
    config::HttpsConfig,
    error::{Error, ErrorKind},
    prelude::*,
    Currency, Price, TradingPair,
};
use bytes::Buf;
use iqhttp::HttpsClient;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

//https://www.imf.org/external/np/fin/data/rms_five.aspx?tsvflag=Y"

/// Base URI for requests to the Coinone API
pub const API_HOST: &str = "www.imf.org";

/// The IMF returns tab seperated data is form that is designed to for

pub struct ImfSdrSource {
    https_client: HttpsClient,
}
/// importing into Excel spreadsheets rather than being machine
/// friendly.record. The TSV data has irregular structure.
/// The strategy here is to find the subsection of data we are
/// looking for by trying to deserialize each row into ImfsdrRow and
/// ignoring deserialization errors. The IMF provides data for the
/// last 5 days but data may no be available for all days.

#[derive(Debug, Deserialize)]
struct ImfsdrRow {
    currency: String,
    price_0: Option<Price>,
    price_1: Option<Price>,
    price_2: Option<Price>,
    price_3: Option<Price>,
    price_4: Option<Price>,
}

impl ImfsdrRow {
    /// Best price is the most recent price. Use
    fn response_from_best_price(&self) -> Option<Response> {
        if let Some(ref price) = self.price_0 {
            return Some(Response { price: *price });
        }
        if let Some(ref price) = self.price_1 {
            return Some(Response { price: *price });
        }
        if let Some(ref price) = self.price_2 {
            return Some(Response { price: *price });
        }
        if let Some(ref price) = self.price_3 {
            return Some(Response { price: *price });
        }
        if let Some(ref price) = self.price_4 {
            return Some(Response { price: *price });
        }
        None
    }
}

impl ImfSdrSource {
    /// Create a new Dunamu source provider
    pub fn new(config: &HttpsConfig) -> Result<Self, Error> {
        let https_client = config.new_client(API_HOST)?;
        Ok(Self { https_client })
    }

    /// Get trading pairs
    pub async fn trading_pairs(&self, pair: &TradingPair) -> Result<Response, Error> {
        if pair.1 != Currency::Sdr {
            fail!(ErrorKind::Currency, "trading pair must be with IMF SDR");
        }

        let uri = format!(
            "https://{}/external/np/fin/data/rms_five.aspx?tsvflag=Y",
            API_HOST
        );

        let body = self
            .https_client
            .get_body(&uri, &Default::default())
            .await?;

        let mut imf_sdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .delimiter(b'\t')
            .from_reader(body.reader());

        let mut response_row: Option<ImfsdrRow> = None;

        for result in imf_sdr.records() {
            let record = result.map_err(|e| {
                format_err!(ErrorKind::Source, "got error with malformed csv: {}", e)
            })?;

            let row: Result<ImfsdrRow, csv::Error> = record.deserialize(None);

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
    /// Price
    pub price: Price,
}

impl TryFrom<ImfsdrRow> for Response {
    type Error = &'static str;
    fn try_from(row: ImfsdrRow) -> Result<Self, Self::Error> {
        match row.response_from_best_price() {
            Some(resp) => Ok(resp),
            None => Err("No price data found for for currency pair"),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::ImfSdrSource;
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
        let quote = block_on(
            ImfSdrSource::new(&Default::default())
                .unwrap()
                .trading_pairs(&pair),
        )
        .unwrap();
        dbg!(&quote);
    }

    // `trading_pairs()` with invalid currency pair
    // #[test]
    // #[ignore]
    // fn trading_pairs_404() {
    //     let pair = "N/A".parse().unwrap();
    //     let quote_err = block_on(ImfSdrSource::new().trading_pairs(&pair))
    //         .err()
    //         .unwrap();

    //    use std::error::Error;
    //    let err: &ErrorCode = quote_err.source().unwrap().downcast_ref().unwrap();

    //     assert_eq!(err, &ErrorCode::InternalError);
    // }
}
