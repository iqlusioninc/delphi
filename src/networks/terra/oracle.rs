//! Terra exchange rate oracle

use super::{denom::Denom, msg::MsgExchangeRateVote, CHAIN_ID, GAS_AMOUNT, MEMO, SCHEMA};
use crate::prelude::*;
use serde_json::json;
use std::{
    convert::{Infallible, TryInto},
    sync::Arc,
    time::Instant,
};
use tokio::sync::Mutex;
use warp::http::StatusCode;

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
        let started_at = Instant::now();
        let msgs = self.get_vote_msgs().await;

        let msg_json = msgs
            .iter()
            .map(|msg| msg.to_json_value(&SCHEMA))
            .collect::<Vec<_>>();

        let tx = json!({
            "chain_id": CHAIN_ID,
            "fee": stdtx::StdFee::for_gas(GAS_AMOUNT),
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

    /// Get oracle vote messages
    async fn get_vote_msgs(self) -> Vec<stdtx::Msg> {
        let mut state = self.0.lock().await;

        // Move all previously unrevealed votes into the result
        let mut msgs = vec![];
        msgs.append(&mut state.unrevealed_votes);

        for denom in Denom::kinds() {
            let exchange_rate = match denom.get_exchange_rate().await {
                Ok(rate) => rate,
                Err(err) => {
                    error!("error getting exchange rate for {}: {}", denom, err);
                    continue;
                }
            };

            let vote_msg = MsgExchangeRateVote {
                denom: *denom,
                exchange_rate: exchange_rate
                    .try_into()
                    .expect("invalid decimal precision (must be 0 or 18"),
                salt: MsgExchangeRateVote::random_salt(),
                feeder: state.feeder,
                validator: state.validator,
            };

            let prevote_msg_stdtx = vote_msg
                .prevote()
                .to_stdtx_msg()
                .expect("can't serialize vote as stdtx");

            msgs.push(prevote_msg_stdtx);

            let vote_msg_stdtx = vote_msg
                .to_stdtx_msg()
                .expect("can't serialize vote as stdtx");

            state.unrevealed_votes.push(vote_msg_stdtx);
        }

        msgs
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
}
