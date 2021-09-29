//! Terra stablecoin project schema
//! <https://terra.money/>

pub mod denom;
pub mod msg;
pub mod oracle;
pub mod proto;

pub use self::{denom::Denom, oracle::ExchangeRateOracle};

use once_cell::sync::Lazy;

/// Memo to include in transactions
pub const MEMO: &str = concat!("delphi/", env!("CARGO_PKG_VERSION"));

/// StdTx schema as parsed from `schema.toml`
static SCHEMA: Lazy<stdtx::amino::Schema> =
    Lazy::new(|| include_str!("terra/schema.toml").parse().unwrap());
