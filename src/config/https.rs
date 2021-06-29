//! HTTPS configuration

use iqhttp::{HttpsClient, Uri};
use serde::{Deserialize, Serialize};

/// Shared HTTPS configuration settings for Delphi sources
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HttpsConfig {
    /// URI to egress proxy
    #[serde(with = "iqhttp::serializers::uri_optional")]
    pub proxy: Option<Uri>,
}

impl HttpsConfig {
    /// Create a new client using this configuration
    pub fn new_client(&self, hostname: impl Into<String>) -> iqhttp::Result<HttpsClient> {
        match &self.proxy {
            Some(proxy_uri) => HttpsClient::new_with_proxy(hostname, proxy_uri.clone()),
            None => Ok(HttpsClient::new(hostname)),
        }
    }
}
