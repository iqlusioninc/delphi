//! Source configuration

use serde::{Deserialize, Serialize};

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
