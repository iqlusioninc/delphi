//! HTTPS configuration

use hyper::Uri;
use serde::{de, ser, Deserialize, Serialize};

/// Shared HTTPS configuration settings for Delphi sources
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HttpsConfig {
    /// URI to egress proxy
    #[serde(
        default,
        serialize_with = "serialize_uri",
        deserialize_with = "deserialize_uri"
    )]
    pub proxy: Option<Uri>,
}

/// Serialize URI
fn serialize_uri<S>(uri: &Option<Uri>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    match uri.as_ref() {
        Some(u) => u.to_string().serialize(serializer),
        None => serializer.serialize_unit(),
    }
}

/// Deserialize URI
fn deserialize_uri<'de, D>(deserializer: D) -> Result<Option<Uri>, D::Error>
where
    D: de::Deserializer<'de>,
{
    use de::Error;
    let uri = String::deserialize(deserializer)?;
    uri.parse().map(Some).map_err(D::Error::custom)
}
