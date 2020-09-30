//! Delphi Config
//!
//! See instructions in `commands.rs` to specify the path to your
//! application's configuration file and/or command-line options
//! for specifying it.

use serde::{Deserialize, Serialize};

/// Delphi Configuration
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DelphiConfig {
    /// Source configuration
    #[serde(default)]
    pub source: SourceConfig,
}

/// Source Configuration
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceConfig {
    /// AlphaVantage
    pub alphavantage: Option<AlphavantageConfig>,
}

/// AlphaVantage Configuration
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AlphavantageConfig {
    /// API key
    pub apikey: String,
}
