//! Terra Oracle Protobuf Definitions
//!
//! Protobuf definitions:
//! <https://github.com/terra-money/core/blob/main/proto/terra/oracle/v1beta1/oracle.proto>
//! <https://github.com/terra-money/core/blob/main/proto/terra/oracle/v1beta1/tx.proto>

use cosmrs::tx::MsgProto;
use cosmrs::proto::cosmos::tx::v1beta1::{AuthInfo, TxBody};
use prost::Message;
use prost_types::Any;

/// MsgAggregateExchangeRatePrevote - struct for message to submit aggregate exchange rate prevote.
#[derive(Clone, PartialEq, Message)]
pub struct MsgAggregateExchangeRatePrevote {
    /// Hash
    #[prost(string, tag = "1")]
    pub hash: String,

    /// Feeder
    #[prost(string, tag = "2")]
    pub feeder: String,

    /// Validator
    #[prost(string, tag = "3")]
    pub validator: String,
}

impl MsgProto for MsgAggregateExchangeRatePrevote {
    const TYPE_URL: &'static str = "/terra.oracle.v1beta1.MsgAggregateExchangeRatePrevote";
}

/// MsgAggregateExchangeRateVote - struct for message to submit aggregate exhcnage rate vote.
#[derive(Clone, PartialEq, Message)]
pub struct MsgAggregateExchangeRateVote {
    /// Salt
    #[prost(string, tag = "1")]
    pub salt: String,

    /// Exchange Rates
    #[prost(string, tag = "2")]
    pub exchange_rates: String,

    /// Feeder
    #[prost(string, tag = "3")]
    pub feeder: String,

    /// Validator
    #[prost(string, tag = "4")]
    pub validator: String,
}

impl MsgProto for MsgAggregateExchangeRateVote {
    const TYPE_URL: &'static str = "/terra.oracle.v1beta1.MsgAggregateExchangeRateVote";
}

/// Request to sign a transaction request
#[derive(Clone, PartialEq, Message)]
pub struct TxSigningRequest {
    /// Requested chain ID
    #[prost(message, tag = "1")]
    pub chain_id: Option<String>,

    /// Messages to include in the transaction.
    #[prost(message, tag = "2")]
    pub msg: Vec<Any>,
}
