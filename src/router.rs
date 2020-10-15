//! HTTP request router (based on warp)
//!
//! Test with:
//!
//! ```text
//! curl -i -X POST -H "Content-Type: application/json" -d '{"network":"columbus-4"}' http://127.0.0.1:23456/oracles/terra
//! ```

use crate::networks::terra;
use warp::Filter;

/// Route incoming requests
pub async fn route() {
    // TODO(tarcieri): configure listen addr/port from config file
    let listen_addr = [127, 0, 0, 1];
    let listen_port = 23456;

    let terra_oracle = terra::ExchangeRateOracle::new();
    let terra_oracle_filter = warp::any().map(move || terra_oracle.clone());

    let app = warp::post()
        .and(warp::path("oracles"))
        .and(warp::path("terra"))
        .and(warp::path::end())
        .and(terra_oracle_filter.clone())
        .and(warp::body::json())
        .and_then(terra::ExchangeRateOracle::handle_request);

    warp::serve(app).run((listen_addr, listen_port)).await;
}
