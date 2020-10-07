//! HTTPS Client

use crate::Error;
use bytes::buf::ext::BufExt;
use hyper::{
    client::{Client, HttpConnector},
    header, Body, Request, Uri,
};
use hyper_rustls::HttpsConnector;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::fmt::{self, Display};
use std::iter::FromIterator;

/// User-Agent to send in HTTP request
pub const USER_AGENT: &str = "iqlusion delphi";

/// HTTPS Client
pub struct HttpsClient {
    http_client: Client<HttpsConnector<HttpConnector>>,
    hostname: String,
}

impl HttpsClient {
    /// Create a newHTTPS client
    pub fn new(hostname: impl Into<String>) -> Self {
        Self {
            http_client: Client::builder().build(HttpsConnector::new()),
            hostname: hostname.into(),
        }
    }

    /// HTTP GET request that gets json
    pub async fn get_json<T>(&self, path: &str, query: &Query) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let uri = query.to_request_uri(&self.hostname, path);

        let mut request = Request::builder()
            .method("GET")
            .uri(&uri)
            .body(Body::empty())?;

        {
            let headers = request.headers_mut();
            headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
            headers.insert(
                header::USER_AGENT,
                format!("{}/{}", USER_AGENT, env!("CARGO_PKG_VERSION"))
                    .parse()
                    .unwrap(),
            );
        }

        let response = self.http_client.request(request).await?;
        let body = hyper::body::aggregate(response.into_body()).await?;
        Ok(serde_json::from_reader(body.reader())?)
    }
}

/// HTTP Query string
/// <https://en.wikipedia.org/wiki/Query_string>
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Query(BTreeMap<String, String>);

impl Query {
    /// Create params
    pub fn new() -> Self {
        Self::default()
    }

    /// Add params
    pub fn add(&mut self, field: impl Into<String>, value: impl Into<String>) -> bool {
        //TODO: return result
        self.0.insert(field.into(), value.into()).is_none()
    }

    /// Compute [`Uri`]
    pub fn to_request_uri(&self, hostname: &str, path: &str) -> Uri {
        let path_and_query = format!("{}?{}", path, self);

        Uri::builder()
            .scheme("https")
            .authority(hostname)
            .path_and_query(path_and_query.as_str())
            .build()
            .unwrap()
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, (field, value)) in self.0.iter().enumerate() {
            write!(f, "{}={}", field, value)?;

            if i < self.0.len() - 1 {
                write!(f, "&")?;
            }
        }

        Ok(())
    }
}

impl<'a> FromIterator<&'a (String, String)> for Query {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a (String, String)>,
    {
        let mut params = Self::new();

        for (field, value) in iter {
            params.add(field, value);
        }

        params
    }
}

#[cfg(test)]
mod tests {
    use super::{FromIterator, Query};

    #[test]
    fn params_to_string() {
        let params = Query::from_iter(&[
            ("foo".to_owned(), "value_1".to_owned()),
            ("bar".to_owned(), "value_2".to_owned()),
        ]);

        let serialized_params = params.to_string();
        assert_eq!(&serialized_params, "bar=value_2&foo=value_1");
    }
}
