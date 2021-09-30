//! Transaction signing requests

use cosmrs::proto::cosmos::tx::v1beta1::{AuthInfo, TxBody};
use prost::Message;

/// Request to sign a transaction request
#[derive(Clone, PartialEq, Message)]
pub struct TxSigningRequest {
    /// Requested chain ID
    #[prost(message, tag = "1")]
    pub chain_id: Option<String>,

    /// body is the processable content of the transaction
    #[prost(message, tag = "2")]
    pub tx_body: Option<TxBody>,

    /// body is the processable content of the transaction
    #[prost(message, tag = "3")]
    pub auth_info: Option<AuthInfo>,
}
