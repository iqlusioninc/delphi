//! Terra stablecoin project schema
//! <https://terra.money/>

pub mod denom;
pub mod msg;
pub mod oracle;

pub use self::{denom::Denom, oracle::ExchangeRateOracle};

use once_cell::sync::Lazy;

/// Chain ID
// TODO(tarcieri): load from config
pub const CHAIN_ID: &str = "columbus-4";

/// Amount of gas to use when voting
pub const GAS_AMOUNT: u64 = 200_000;

/// Memo to include in transactions
pub const MEMO: &str = concat!("delphi/", env!("CARGO_PKG_VERSION"));

/// StdTx schema as parsed from `schema.toml`
static SCHEMA: Lazy<stdtx::Schema> =
    Lazy::new(|| include_str!("terra/schema.toml").parse().unwrap());
