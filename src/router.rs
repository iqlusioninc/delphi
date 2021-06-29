//! HTTP request router (based on warp)
//!
//! Test with:
//!
//! ```text
//! curl -i -X POST -H "Content-Type: application/json" -d '{"network":"columbus-4"}' http://127.0.0.1:23456/oracles/terra
//! ```

use crate::{config::listen::Protocol, error::Error, networks::terra, prelude::*};
use serde::Deserialize;
use std::convert::Infallible;
use tendermint::chain;
use tendermint_rpc::endpoint::{broadcast::tx_commit, status::SyncInfo};
use warp::Filter;

/// HTTP request router
#[derive(Clone)]
pub struct Router {
    /// Address to listen on
    addr: ([u8; 4], u16),

    /// Protocol to listen on
    protocol: Protocol,

    /// Terra oracle
    terra_oracle: terra::ExchangeRateOracle,
}

impl Router {
    /// Initialize the router from the config
    pub fn init() -> Result<Router, Error> {
        let config = APP.config();
        let addr = (config.listen.addr.octets(), config.listen.port);
        let protocol = config.listen.protocol;
        let terra_oracle = terra::ExchangeRateOracle::new(&config)?;
        Ok(Self {
            addr,
            protocol,
            terra_oracle,
        })
    }

    /// Route incoming requests
    pub async fn route(self) {
        let addr = self.addr;
        let protocol = self.protocol;
        let terra_oracle_filter = warp::any().map(move || self.terra_oracle.clone());

        let app = warp::post()
            .and(warp::path("oracle"))
            .and(warp::path::end())
            .and(terra_oracle_filter.clone())
            .and(warp::body::json())
            .and_then(oracle_request);

        match protocol {
            Protocol::Http => warp::serve(app).run(addr).await,
        }
    }
}

/// `POST /oracle` - handle incoming oracle requests
///
/// This endpoint is intended to be triggered by Tendermint KMS
pub async fn oracle_request(
    oracle: terra::ExchangeRateOracle,
    req: Request,
) -> Result<impl warp::Reply, Infallible> {
    // TODO(tarcieri): dispatch incoming requests based on chain ID
    oracle.handle_request(req).await
}

/// Incoming oracle requests from Tendermint KMS (serialized as JSON)
#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    /// Chain ID
    pub network: chain::Id,

    /// Arbitrary context string to pass to transaction source
    #[serde(default)]
    pub context: String,

    /// Network status
    pub status: Option<SyncInfo>,

    /// Response from last signed TX (if available)
    pub last_tx_response: Option<tx_commit::Response>,
}
