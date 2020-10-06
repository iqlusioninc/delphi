//! Terra exchange rate oracle

use super::{
    denom::Denom,
    msg::{self, MsgAggregateExchangeRateVote},
    GAS_AMOUNT, MEMO, SCHEMA,
};
use crate::{application::app_config, config::TerraConfig, prelude::*, sources::Sources};
use futures::future::join_all;
use serde_json::json;
use std::{convert::Infallible, sync::Arc, time::Instant};
use tokio::sync::Mutex;
use warp::http::StatusCode;

/// Terra exchange rate oracle
#[derive(Clone)]
pub struct ExchangeRateOracle(Arc<Mutex<OracleState>>);

impl ExchangeRateOracle {
    /// Create a new [`ExchangeRateOracle`]
    pub fn new(feeder: stdtx::Address, validator: stdtx::Address) -> Self {
        let state = {
            let config = app_config();
            let sources = Sources::new(&config);
            let terra_config = app_config().network.terra.clone().expect("no Terra config");
            OracleState::new(feeder, validator, sources, terra_config)
        };

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
        let chain_id = self.get_chain_id().await;
        let msgs = self.get_vote_msgs().await;

        let msg_json = msgs
            .iter()
            .map(|msg| msg.to_json_value(&SCHEMA))
            .collect::<Vec<_>>();

        let tx = json!({
            "chain_id": chain_id,
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

    /// Get the chain ID
    async fn get_chain_id(&self) -> String {
        let state = self.0.lock().await;
        state.terra_config.chain_id.clone()
    }

    /// Get oracle vote messages
    async fn get_vote_msgs(&self) -> Vec<stdtx::Msg> {
        let mut state = self.0.lock().await;
        let mut exchange_rates = msg::ExchangeRates::new();

        let mut exchange_rate_fut = vec![];
        for denom in Denom::kinds() {
            exchange_rate_fut.push(denom.get_exchange_rate(&state.sources))
        }
        let rates = join_all(exchange_rate_fut).await;

        for (rate, denom) in rates.iter().zip(Denom::kinds()) {
            match rate {
                Ok(rate) => exchange_rates.add(*denom, *rate).expect("duplicate denom"),
                Err(err) => {
                    error!("error getting exchange rate for {}: {}", denom, err);
                    continue;
                }
            };
        }

        dbg!(&exchange_rates);

        // Move all previously unrevealed votes into the result
        let mut msgs = vec![];
        msgs.append(&mut state.unrevealed_votes);

        let vote_msg = MsgAggregateExchangeRateVote {
            exchange_rates,
            salt: MsgAggregateExchangeRateVote::random_salt(),
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

        msgs
    }
}

/// Inner (synchronized) oracle state
struct OracleState {
    /// Feeder address
    feeder: stdtx::Address,

    /// Validator address
    validator: stdtx::Address,

    /// Sources
    sources: Sources,

    /// Terra network configuration
    terra_config: TerraConfig,

    /// Previously unrevealed votes
    unrevealed_votes: Vec<stdtx::Msg>,
}

impl OracleState {
    /// Initialize oracle state
    fn new(
        feeder: stdtx::Address,
        validator: stdtx::Address,
        sources: Sources,
        terra_config: TerraConfig,
    ) -> Self {
        Self {
            unrevealed_votes: vec![],
            feeder,
            validator,
            sources,
            terra_config,
        }
    }
}
