//! Error Reporting

use datadog::send_event;
use datadog::Error;
use datadog::Event;
use std::collections::BTreeMap;
use std::env;

///Sending error message to datadog
pub async fn datadog_err(message: String) -> Result<(), Error> {
    let dd_api_key = env::var("DD_API_KEY").unwrap();
    let mut ddtags = BTreeMap::new();
    ddtags.insert("env".to_owned(), "staging".to_owned());
    ddtags.insert("user".to_owned(), "deplhi".to_owned());

    let event = Event {
        ddsource: "delphi".to_owned(),
        service: "delphi".to_owned(),
        ddtags: ddtags,
        hostname: "127.0.0.1".to_owned(),
        message,
    };

    send_event(&event, dd_api_key).await
}
