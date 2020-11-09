//! Listen config

use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

/// Default port number (38°28′56″ N, 22°30′4″ E)
pub const DEFAULT_PORT: u16 = 3822;

/// Listen config: control how Delphi listens on the network for requests
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ListenConfig {
    /// IPv4 address to listen on
    // TODO(tarcieri): IPv6
    pub addr: Ipv4Addr,

    /// Port to listen on
    pub port: u16,

    /// Protocol to listen on
    pub protocol: Protocol,
}

impl Default for ListenConfig {
    fn default() -> Self {
        Self {
            addr: Ipv4Addr::new(127, 0, 0, 1),
            port: DEFAULT_PORT,
            protocol: Protocol::default(),
        }
    }
}

/// Protocol to listen on
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Protocol {
    /// Plaintext HTTP
    // TODO(tarcieri): HTTPS, gRPC
    #[serde(rename = "http")]
    Http,
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::Http
    }
}
