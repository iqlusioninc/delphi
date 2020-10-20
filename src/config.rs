//! Delphi Config
//!
//! See instructions in `commands.rs` to specify the path to your
//! application's configuration file and/or command-line options
//! for specifying it.

pub mod https;
pub mod listen;
pub mod network;
pub mod source;

pub use self::{
    https::HttpsConfig, listen::ListenConfig, network::NetworkConfig, source::SourceConfig,
};

use serde::{Deserialize, Serialize};

/// Delphi Configuration
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DelphiConfig {
    /// Listen configuration
    #[serde(default)]
    pub listen: ListenConfig,

    /// HTTPS client configuration
    #[serde(default)]
    pub https: HttpsConfig,

    /// Network (i.e. chain) configuration
    #[serde(default)]
    pub network: NetworkConfig,

    /// Source configuration
    #[serde(default)]
    pub source: SourceConfig,
}
