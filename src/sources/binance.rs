//! Binance Source Provider
//! <https://binance.com/>
//!
//! Uses the nash.io openLimits client:
//! <https://github.com/nash-io/openlimits>

use super::Pair;
use crate::error::{Error, ErrorKind};
use openlimits::exchange_info::MarketPairHandle;

/// Source provider for Binance
pub struct BinanceSource {
    client: openlimits::binance::Binance,
}

impl BinanceSource {
    /// Create a new Binance source provider
    pub async fn new() -> Self {
        let client = openlimits::binance::Binance::new(false).await;
        Self { client }
    }

    /// Create a new Binance source provider which talks to the sandbox
    pub async fn new_sandboxed() -> Self {
        let client = openlimits::binance::Binance::new(true).await;
        Self { client }
    }

    /// Get trading pairs
    pub fn trading_pairs(&self, pair: &Pair) -> Result<Response, Error> {
        self.client
            .get_pair(&pair.to_string())
            .map(Into::into)
            .map_err(|e| ErrorKind::Source.context(e).into())
    }
}

/// Binance response
pub struct Response {}

impl From<MarketPairHandle> for Response {
    fn from(_: MarketPairHandle) -> Response {
        Response {}
    }
}
