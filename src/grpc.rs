//!

use tonic::{transport::Server, Request, Response, Status};

use delphi_rpc::price_oracle_server::{PriceOracle, PriceOracleServer};
use delphi_rpc::{PriceRequest, PriceResponse};

pub mod delphi_rpc {
    tonic::include_proto!("delphi"); // The string specified here must match the proto package name
}
