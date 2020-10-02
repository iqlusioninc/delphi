//! Terra message types

// TODO(tarcieri): autogenerate this from the schema? (possibly after proto migration)

use super::{Denom, SCHEMA};
use crate::error::Error;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sha2::{Digest, Sha256};
use stdtx::{Address, Decimal};
use subtle_encoding::hex;

/// Terra Oracle Vote Message (`oracle/MsgExchangeRateVote`)
///
/// <https://docs.terra.money/dev/spec-oracle.html#msgexchangeratevote>
#[derive(Clone, Debug)]
pub struct MsgExchangeRateVote {
    /// Exchange rate voted on. Negative values are an abstain vote.
    pub exchange_rate: Decimal,

    /// Salt for commit reveal protocol
    pub salt: String,

    /// Denom for Oracle Vote
    pub denom: Denom,

    /// Origin of the Feed Msg
    pub feeder: Address,

    /// Validator voting on behalf of
    pub validator: Address,
}

impl MsgExchangeRateVote {
    /// Get a random salt value
    pub fn random_salt() -> String {
        thread_rng().sample_iter(&Alphanumeric).take(4).collect()
    }

    /// Simple builder for an `oracle/MsgExchangeRateVote` message
    pub fn to_stdtx_msg(&self) -> Result<stdtx::Msg, Error> {
        Ok(
            stdtx::msg::Builder::new(&SCHEMA, "oracle/MsgExchangeRateVote")?
                .decimal("exchange_rate", self.exchange_rate)?
                .string("salt", &self.salt)?
                .string("denom", self.denom.as_str())?
                .acc_address("feeder", self.feeder)?
                .val_address("validator", self.validator)?
                .to_msg(),
        )
    }

    /// Compute prevote from this vote
    pub fn prevote(&self) -> MsgExchangeRatePrevote {
        MsgExchangeRatePrevote {
            hash: self.generate_vote_hash(),
            denom: self.denom,
            feeder: self.feeder,
            validator: self.validator,
        }
    }

    /// Generate hex encoded truncated SHA-256 of vote. Needed to generate prevote
    fn generate_vote_hash(&self) -> String {
        let data = format!(
            "{}:{}:{}:{}",
            self.salt,
            self.exchange_rate,
            self.denom.as_str(),
            self.validator.to_bech32("terravaloper"),
        );

        // Tendermint truncated sha256
        let digest = Sha256::digest(data.as_bytes());
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&digest[..20]);

        // Should always succeed.
        String::from_utf8(hex::encode(bytes)).unwrap()
    }
}

/// Terra Oracle Prevote Message (`oracle/MsgExchangeRatePrevote`)
///
/// <https://docs.terra.money/dev/spec-oracle.html#msgexchangerateprevote>
#[derive(Clone, Debug)]
pub struct MsgExchangeRatePrevote {
    /// Commitment to future vote
    pub hash: String,

    /// Denom to commit for
    pub denom: Denom,

    /// Origin Address for vote
    pub feeder: Address,

    /// Validator voting on behalf of
    pub validator: Address,
}

impl MsgExchangeRatePrevote {
    /// Simple builder for an `oracle/MsgExchangeRatePrevote` message
    pub fn to_stdtx_msg(&self) -> Result<stdtx::Msg, Error> {
        Ok(
            stdtx::msg::Builder::new(&SCHEMA, "oracle/MsgExchangeRatePrevote")?
                .string("hash", &self.hash)?
                .string("denom", self.denom.as_str())?
                .acc_address("feeder", self.feeder)?
                .val_address("validator", self.validator)?
                .to_msg(),
        )
    }
}
