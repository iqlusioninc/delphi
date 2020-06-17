//! Terra stablecoin project schema
//! <https://terra.money/>

pub mod msg;

use crate::{error::Error, prelude::*};
use msg::MsgExchangeRateVote;
use once_cell::sync::Lazy;
use serde_json::json;
use std::{
    convert::Infallible,
    sync::{Arc, Mutex},
    time::Instant,
};
use stdtx::Decimal;
use warp::http::StatusCode;

/// Chain ID
// TODO(tarcieri): load from config
pub const CHAIN_ID: &str = "columbus-3";

/// Amount of gas to use when voting
pub const GAS_AMOUNT: u64 = 200000;

/// Memo to include in transactions
pub const MEMO: &str = concat!("delphi/", env!("CARGO_PKG_VERSION"));

/// StdTx schema as parsed from `schema.toml`
static SCHEMA: Lazy<stdtx::Schema> =
    Lazy::new(|| include_str!("terra/schema.toml").parse().unwrap());

/// Terra exchange rate oracle
#[derive(Clone)]
pub struct ExchangeRateOracle(Arc<Mutex<OracleState>>);

impl ExchangeRateOracle {
    /// Create a new [`ExchangeRateOracle`]
    pub fn new(feeder: stdtx::Address, validator: stdtx::Address) -> Self {
        let state = OracleState::new(feeder, validator);
        ExchangeRateOracle(Arc::new(Mutex::new(state)))
    }

    /// Handle an incoming oracle request, providing a set of transactions to
    /// respond with.
    ///
    /// Test with:
    ///
    /// ```text
    /// $ curl --request POST http://127.0.0.1:23456/oracles/terra
    /// ```
    pub async fn handle_request(self) -> Result<impl warp::Reply, Infallible> {
        let mut state = self.0.lock().unwrap();

        match state.get_vote_messages() {
            Ok(msgs) => {
                let started_at = Instant::now();

                let msg_json = msgs
                    .iter()
                    .map(|msg| msg.to_json_value(&SCHEMA))
                    .collect::<Vec<_>>();

                let tx = json!({
                    "chain_id": CHAIN_ID,
                    "fee": stdtx::StdFee::for_gas(GAS_AMOUNT).to_json_value(),
                    "memo": MEMO,
                    "msgs": msg_json,
                });

                let response = json!({
                    "status": "ok",
                    "tx": vec![tx]
                });

                info!("t={:?}", Instant::now().duration_since(started_at));

                Ok(warp::reply::with_status(
                    warp::reply::json(&response),
                    StatusCode::OK,
                ))
            }
            Err(e) => {
                error!("couldn't get vote messages: {}", e);

                let response = json!({
                    "status": "error",
                    "msg": e.to_string()
                });

                Ok(warp::reply::with_status(
                    warp::reply::json(&response),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        }
    }
}

/// Inner (synchronized) oracle state
struct OracleState {
    /// Feeder address
    feeder: stdtx::Address,

    /// Validator address
    validator: stdtx::Address,

    /// Previously unrevealed votes
    unrevealed_votes: Vec<stdtx::Msg>,
}

impl OracleState {
    /// Initialize oracle state
    fn new(feeder: stdtx::Address, validator: stdtx::Address) -> Self {
        Self {
            unrevealed_votes: vec![],
            feeder,
            validator,
        }
    }

    /// Get the next round of oracle vote messages
    fn get_vote_messages(&mut self) -> Result<Vec<stdtx::Msg>, Error> {
        // Move all previously unrevealed votes into the result
        let mut result = vec![];
        result.append(&mut self.unrevealed_votes);

        for denom in Denom::kinds() {
            let vote_msg = self.get_vote_for_denom(*denom)?;
            let prevote_msg = vote_msg.prevote();
            self.unrevealed_votes.push(vote_msg.to_stdtx_msg()?);
            result.push(prevote_msg.to_stdtx_msg()?);
        }

        Ok(result)
    }

    /// Create a vote message for a particular denom
    fn get_vote_for_denom(&mut self, denom: Denom) -> Result<MsgExchangeRateVote, Error> {
        // TODO(tarcieri): compute non-abstain votes for each denom
        Ok(MsgExchangeRateVote {
            denom,
            exchange_rate: Decimal::from(-1i8),
            salt: MsgExchangeRateVote::random_salt(),
            feeder: self.feeder.clone(),
            validator: self.validator.clone(),
        })
    }
}

/// Denomination
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Denom {
    /// Korean  Wan
    UKRW,

    /// Singaporean Dollar
    UMNT,

    /// IMF Special Drawing Rights
    USDR,

    /// US Dollars
    UUSD,
}

impl Denom {
    /// Get a slice of the [`Denom`] kinds
    pub fn kinds() -> &'static [Denom] {
        &[Denom::UKRW, Denom::UMNT, Denom::USDR, Denom::UUSD]
    }

    /// Get the code corresponding to a [`Denom`]
    pub fn as_str(&self) -> &str {
        match *self {
            Denom::UKRW => "ukrw",
            Denom::UMNT => "umnt",
            Denom::USDR => "usdr",
            Denom::UUSD => "uusd",
        }
    }
}
