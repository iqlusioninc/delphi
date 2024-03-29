//! Delphi Oracle Service

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, trivial_casts, unused_qualifications)]

pub mod application;
pub mod commands;
pub mod config;
pub mod currency;
pub mod error;
pub mod networks;
pub mod prelude;
pub mod price;
pub mod protos;
pub mod router;
pub mod sources;
pub mod trading_pair;

pub use self::{
    currency::Currency,
    error::{Error, ErrorKind},
    price::{Price, PriceQuantity},
    trading_pair::TradingPair,
};

pub use std::collections::{btree_map as map, BTreeMap as Map};

/// User-Agent to send in HTTP request
pub const USER_AGENT: &str = concat!("iqlusion delphi/", env!("CARGO_PKG_VERSION"));
