//! Network configuration

use crate::networks::terra::Denom;
use serde::{Deserialize, Serialize};
use stdtx::amino_types::{Coin, StdFee};

/// Network/chain specific configuration
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkConfig {
    /// Terra configuration
    pub terra: Option<TerraConfig>,
}

/// Terra configuration
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TerraConfig {
    /// Terra chain id
    pub chain_id: String,

    /// Feeder address (Bech32)
    pub feeder: String,

    /// Validator address (Bech32)
    pub validator: String,

    /// Oracle transaction fee
    #[serde(default)]
    pub fee: TerraOracleFee,

    /// Timeout for an oracle vote in seconds (default 10)
    pub timeout_secs: Option<u64>,
}

/// Terra oracle fee configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TerraOracleFee {
    /// Fee denomination
    pub denom: Denom,

    /// Fee amount
    pub amount: u64,

    /// Gas amount
    pub gas: u64,
}

impl Default for TerraOracleFee {
    fn default() -> Self {
        Self {
            denom: Denom::UKRW,
            amount: 356_100,
            gas: 200_000,
        }
    }
}

impl From<&TerraOracleFee> for StdFee {
    fn from(fee: &TerraOracleFee) -> StdFee {
        StdFee {
            amount: vec![Coin {
                denom: fee.denom.to_string(),
                amount: fee.amount.to_string(),
            }],
            gas: fee.gas,
        }
    }
}
