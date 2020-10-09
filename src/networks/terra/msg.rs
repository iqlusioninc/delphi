//! Terra message types

// TODO(tarcieri): autogenerate this from the schema? (possibly after proto migration)

use super::{Denom, SCHEMA};
use crate::{
    error::{Error, ErrorKind},
    prelude::*,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap as Map,
    convert::TryFrom,
    fmt::{self, Display},
};
use stdtx::{Address, Decimal};

/// Truncated SHA-256 hash to include in a pre-vote
pub type Hash = [u8; 20];

/// Message type name for [`MsgAggregateExchangeRateVote`]
pub const VOTE_MSG_NAME: &str = "oracle/MsgAggregateExchangeRateVote";

/// Message type name for [`MsgAggregateExchangeRatePrevote`]
pub const PREVOTE_MSG_NAME: &str = "oracle/MsgAggregateExchangeRatePrevote";

/// Terra Oracle Aggregate Vote Message (`oracle/MsgAggregateExchangeRateVote`)
/// <https://docs.terra.money/dev/spec-oracle.html#msgaggregateexchangeratevote>
#[derive(Clone, Debug)]
pub struct MsgAggregateExchangeRateVote {
    /// Exchange rates to be voted on. Negative values are an abstain vote.
    pub exchange_rates: ExchangeRates,

    /// Salt for commit reveal protocol
    pub salt: String,

    /// Origin of the Feed Msg
    pub feeder: Address,

    /// Validator voting on behalf of
    pub validator: Address,
}

impl MsgAggregateExchangeRateVote {
    /// Get a random salt value
    pub fn random_salt() -> String {
        thread_rng().sample_iter(&Alphanumeric).take(4).collect()
    }

    /// Simple builder for an `oracle/MsgAggregateExchangeRateVote` message
    pub fn to_stdtx_msg(&self) -> Result<stdtx::Msg, Error> {
        Ok(stdtx::msg::Builder::new(&SCHEMA, VOTE_MSG_NAME)?
            .string("exchange_rates", self.exchange_rates.to_string())?
            .string("salt", &self.salt)?
            .acc_address("feeder", self.feeder)?
            .val_address("validator", self.validator)?
            .to_msg())
    }

    /// Compute prevote from this vote
    pub fn prevote(&self) -> MsgAggregateExchangeRatePrevote {
        MsgAggregateExchangeRatePrevote {
            hash: self.generate_vote_hash(),
            feeder: self.feeder,
            validator: self.validator,
        }
    }

    /// Generate hex encoded truncated SHA-256 of vote. Needed to generate prevote
    fn generate_vote_hash(&self) -> Hash {
        let data = format!(
            "{}:{}:{}",
            self.salt,
            self.exchange_rates.to_string(),
            self.validator.to_bech32("terravaloper"),
        );

        // Tendermint truncated sha256
        let digest = Sha256::digest(data.as_bytes());
        Hash::try_from(&digest[..20]).unwrap()
    }
}

/// Terra Oracle Aggregate Prevote Message (`oracle/MsgAggregateExchangeRatePrevote`)
/// <https://docs.terra.money/dev/spec-oracle.html#msgaggregateexchangerateprevote>
#[derive(Clone, Debug)]
pub struct MsgAggregateExchangeRatePrevote {
    /// Commitment to future vote
    pub hash: Hash,

    /// Origin Address for vote
    pub feeder: Address,

    /// Validator voting on behalf of
    pub validator: Address,
}

impl MsgAggregateExchangeRatePrevote {
    /// Simple builder for an `oracle/MsgAggregateExchangeRatePrevote` message
    pub fn to_stdtx_msg(&self) -> Result<stdtx::Msg, Error> {
        Ok(stdtx::msg::Builder::new(&SCHEMA, PREVOTE_MSG_NAME)?
            .bytes("hash", self.hash.as_ref())?
            .acc_address("feeder", self.feeder)?
            .val_address("validator", self.validator)?
            .to_msg())
    }
}

/// Exchange rates
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExchangeRates(Map<Denom, Decimal>);

impl ExchangeRates {
    /// Create a new set of exchange rates
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new set of exchange rates from an iterator over
    /// `(Denom,Decimal)` tuples
    // NOTE: can't use `FromIterator` here because of the `Result`
    pub fn from_exchange_rates<'a, I>(iter: I) -> Result<Self, Error>
    where
        I: Iterator<Item = &'a (Denom, Decimal)>,
    {
        let mut exchange_rates = ExchangeRates::new();

        for &(denom, rate) in iter {
            exchange_rates.add(denom, rate)?;
        }

        Ok(exchange_rates)
    }

    /// Add an exchange rate
    pub fn add(&mut self, denom: Denom, rate: Decimal) -> Result<(), Error> {
        let duplicate = self.0.insert(denom, rate).is_some();

        ensure!(
            !duplicate,
            ErrorKind::Currency,
            "duplicate exchange rate for denom: {}",
            denom
        );

        Ok(())
    }
}

impl Display for ExchangeRates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, (denom, rate)) in self.0.iter().enumerate() {
            write!(f, "{}{}", rate, denom)?;

            if i < self.0.len() - 1 {
                write!(f, ",")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Denom, ExchangeRates};

    #[test]
    fn exchange_rate_to_string() {
        let exchange_rates = ExchangeRates::from_exchange_rates(
            [
                (Denom::UUSD, "1".parse().unwrap()),
                (Denom::USDR, "1".parse().unwrap()),
                (Denom::UMNT, "888".parse().unwrap()),
                (Denom::UKRW, "362".parse().unwrap()),
            ]
            .iter(),
        )
        .unwrap();

        let serialized_rates = exchange_rates.to_string();
        assert_eq!(
            &serialized_rates,
            "362.000000000000000000ukrw,\
            888.000000000000000000umnt,\
            1.000000000000000000usdr,\
            1.000000000000000000uusd"
        );
    }
}
