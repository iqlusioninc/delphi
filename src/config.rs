//! Delphi Config
//!
//! See instructions in `commands.rs` to specify the path to your
//! application's configuration file and/or command-line options
//! for specifying it.

pub mod https;
pub mod network;
pub mod source;

pub use self::{https::HttpsConfig, network::NetworkConfig, source::SourceConfig};

use serde::{Deserialize, Serialize};

/// Delphi Configuration
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DelphiConfig {
    /// HTTPS configuration
    #[serde(default)]
    pub https: HttpsConfig,

    /// Network configuration
    #[serde(default)]
    pub network: NetworkConfig,

    /// Source configuration
    #[serde(default)]
    pub source: SourceConfig,
}
