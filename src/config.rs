//! Delphi Config
//!
//! See instructions in `commands.rs` to specify the path to your
//! application's configuration file and/or command-line options
//! for specifying it.

pub mod network;
pub mod source;

pub use self::{network::NetworkConfig, source::SourceConfig};

use serde::{Deserialize, Serialize};

/// Delphi Configuration
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DelphiConfig {
    /// Network configuration
    #[serde(default)]
    pub network: NetworkConfig,

    /// Source configuration
    #[serde(default)]
    pub source: SourceConfig,
}
