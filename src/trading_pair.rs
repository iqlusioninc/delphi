//! Trading pairs

use crate::{prelude::*, Currency, Error, ErrorKind};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{de, ser, Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

/// Trading pairs
pub struct TradingPair(pub Currency, pub Currency);

impl TradingPair {
    /// Percent encode this pair (for inclusion in a URL)
    pub fn percent_encode(&self) -> String {
        utf8_percent_encode(&self.to_string(), NON_ALPHANUMERIC).to_string()
    }
}

impl Display for TradingPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.0, self.1)
    }
}

impl FromStr for TradingPair {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let pair: Vec<_> = s.split('/').collect();

        if pair.len() != 2 {
            fail!(ErrorKind::Parse, "malformed trading pair: {}", s);
        }

        Ok(TradingPair(pair[0].parse()?, pair[1].parse()?))
    }
}

impl<'de> Deserialize<'de> for TradingPair {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use de::Error;
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

impl Serialize for TradingPair {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}
