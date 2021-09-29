//! Terra Oracle Protobuf Definitions
//!
//! Protobuf definitions:
//! <https://github.com/terra-money/core/blob/main/proto/terra/oracle/v1beta1/oracle.proto>

use cosmrs::tx::MsgProto;
use prost::Message;

/// Struct for aggregate prevoting on the ExchangeRateVote.
///
/// The purpose of aggregate prevote is to hide vote exchange rates with hash
/// which is formatted as hex string in
/// `SHA256("{salt}:{exchange rate}{denom},...,{exchange rate}{denom}:{voter}")`
#[derive(Clone, PartialEq, Message)]
pub struct AggregateExchangeRatePrevote {
    /// Commitment to future vote
    #[prost(string, tag = "1")]
    pub hash: String,

    /// Origin Address for vote
    #[prost(string, tag = "2")]
    pub voter: String,

    /// Submit block
    #[prost(uint64, tag = "3")]
    pub submit_block: u64,
}

impl MsgProto for AggregateExchangeRatePrevote {
    const TYPE_URL: &'static str = "/terra.oracle.v1beta1.AggregateExchangeRatePrevote";
}

/// MsgAggregateExchangeRateVote - struct for voting on
/// the exchange rates of Luna denominated in various Terra assets.
#[derive(Clone, PartialEq, Message)]
pub struct AggregateExchangeRateVote {
    /// Exchange rate tuples
    #[prost(message, repeated, tag = "1")]
    pub exchange_rate_tuples: Vec<ExchangeRateTuple>,

    /// Origin Address for vote
    #[prost(string, tag = "2")]
    pub voter: String,
}

impl MsgProto for AggregateExchangeRateVote {
    const TYPE_URL: &'static str = "/terra.oracle.v1beta1.AggregateExchangeRateVote";
}

/// ExchangeRateTuple - struct to store interpreted exchange rates data to store
#[derive(Clone, PartialEq, Message)]
pub struct ExchangeRateTuple {
    /// Denomination
    #[prost(string, tag = "1")]
    pub denom: String,

    /// Exchange rate
    #[prost(string, tag = "2")]
    pub exchange_rate: String,
}
