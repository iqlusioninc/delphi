//! Terra exchange rate oracle

use super::{
    denom::Denom,
    msg::{self, MsgAggregateExchangeRateVote},
    MEMO, SCHEMA,
};
use crate::{
    config::DelphiConfig, error_reporting::datadog_err, prelude::*, router::Request,
    sources::Sources, Error,
};
use futures::future::join_all;
use serde_json::json;
use std::{
    convert::Infallible,
    sync::Arc,
    time::{Duration, Instant},
};
use stdtx::{amino_types::StdFee, Address};
use tendermint_rpc::endpoint::broadcast::tx_commit;
use tokio::{sync::Mutex, time::timeout};
use warp::http::StatusCode;

/// Number of seconds to wait for an oracle vote complete
pub const DEFAULT_TIMEOUT_SECS: u64 = 10;

/// Terra exchange rate oracle
#[derive(Clone)]
pub struct ExchangeRateOracle(Arc<Mutex<OracleState>>);

impl ExchangeRateOracle {
    /// Create a new [`ExchangeRateOracle`]
    pub fn new(config: &DelphiConfig) -> Result<Self, Error> {
        let state = OracleState::new(&config)?;
        Ok(ExchangeRateOracle(Arc::new(Mutex::new(state))))
    }

    /// Handle an incoming oracle request, providing a set of transactions to
    /// respond with.
    pub async fn handle_request(self, req: Request) -> Result<impl warp::Reply, Infallible> {
        let chain_id = self.get_chain_id().await;
        let msgs = self.get_vote_msgs(req.last_tx_response).await;

        let response = if msgs.is_empty() {
            json!({"status": "ok"})
        } else {
            let msg_json = msgs
                .iter()
                .map(|msg| msg.to_json_value(&SCHEMA))
                .collect::<Vec<_>>();

            let tx = json!({
                "chain_id": chain_id,
                "fee": self.oracle_fee().await,
                "memo": MEMO,
                "msgs": msg_json,
            });

            json!({
                "status": "ok",
                "tx": tx
            })
        };

        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            StatusCode::OK,
        ))
    }

    /// Get the chain ID
    async fn get_chain_id(&self) -> String {
        let state = self.0.lock().await;
        state.chain_id.clone()
    }

    /// Get oracle vote messages
    async fn get_vote_msgs(
        &self,
        last_tx_response: Option<tx_commit::Response>,
    ) -> Vec<stdtx::Msg> {
        let started_at = Instant::now();
        let mut state = self.0.lock().await;
        let mut exchange_rates = msg::ExchangeRates::new();
        let mut exchange_rate_fut = vec![];

        for denom in Denom::kinds() {
            exchange_rate_fut.push(denom.get_exchange_rate(&state.sources))
        }

        let rates = match timeout(state.timeout, join_all(exchange_rate_fut)).await {
            Ok(res) => res,
            Err(e) => {
                warn!("oracle vote timed out after {:?}: {}", state.timeout, e);

                match datadog_err(format!(
                    "oracle vote timed out after {:?}: {}",
                    state.timeout, e
                ))
                .await
                {
                    Ok(_) => (),
                    Err(e) => warn!("Datadog event failed {:?}", e),
                }

                return vec![];
            }
        };

        for (rate, denom) in rates.iter().zip(Denom::kinds()) {
            match rate {
                Ok(rate) => exchange_rates.add(*denom, *rate).expect("duplicate denom"),
                Err(err) => {
                    error!("error getting exchange rate for {}: {}", denom, err);

                    match datadog_err(format!(
                        "error getting exchange rate for {}: {}",
                        denom, err
                    ))
                    .await
                    {
                        Ok(_) => (),
                        Err(e) => warn!("Datadog event failed {:?}", e),
                    }

                    continue;
                }
            };
        }

        info!(
            "voting {} ({:?})",
            exchange_rates
                .iter()
                .map(|(denom, decimal)| format!("{}={}", denom, decimal))
                .collect::<Vec<_>>()
                .join(", "),
            Instant::now().duration_since(started_at)
        );

        // Move all previously unrevealed votes into the result
        let mut msgs = vec![];

        if let Some(vote) = state.unrevealed_vote.take() {
            // Determine if the last transaction we sent was successful
            let last_tx_success = last_tx_response
                .map(|tx| tx.check_tx.code.is_ok() && tx.deliver_tx.code.is_ok())
                .unwrap_or(false);

            if last_tx_success {
                // Only include the previous vote if we succeeded in publishing
                // an oracle prevote. Otherwise DeliverTx fails because we
                // don't have a corresponding prevote
                msgs.push(vote);
            }
        }

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

        state.unrevealed_vote = Some(vote_msg_stdtx);

        msgs
    }

    /// Compute the oracle fee
    pub async fn oracle_fee(&self) -> StdFee {
        let state = self.0.lock().await;
        state.fee.clone()
    }
}

/// Inner (synchronized) oracle state
struct OracleState {
    /// Chain ID
    chain_id: String,

    /// Feeder address
    feeder: Address,

    /// Validator address
    validator: Address,

    /// Fee
    fee: StdFee,

    /// Sources
    sources: Sources,

    /// Timeout
    timeout: Duration,

    /// Previously unrevealed vote
    unrevealed_vote: Option<stdtx::Msg>,
}

impl OracleState {
    /// Initialize oracle state
    fn new(config: &DelphiConfig) -> Result<Self, Error> {
        let terra_config = config
            .network
            .terra
            .as_ref()
            .expect("missing [networks.terra] config");

        let feeder = Address::from_bech32(&terra_config.feeder)
            .expect("invalid terra feeder config")
            .1;

        let validator = Address::from_bech32(&terra_config.validator)
            .expect("invalid terra validator config")
            .1;

        let fee = StdFee::from(&terra_config.fee);
        let sources = Sources::new(config)?;
        let timeout =
            Duration::from_secs(terra_config.timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS));

        Ok(Self {
            chain_id: terra_config.chain_id.to_owned(),
            feeder,
            validator,
            fee,
            sources,
            timeout,
            unrevealed_vote: None,
        })
    }
}
