//! Binance Source Provider
//! <https://binance.com/>

use super::{Currency, Pair, USER_AGENT};
use crate::{
    error::{Error, ErrorKind},
    prelude::*,
};
use bytes::buf::ext::BufExt;
use hyper::{
    client::{Client, HttpConnector},
    header, Body, Request, Uri,
};
use hyper_rustls::HttpsConnector;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    fmt::{self, Display},
    str::FromStr,
};

/// Hostname for the Binance API
pub const API_HOST: &str = "api.binance.com";

/// Source provider for Binance
pub struct BinanceSource {
    http_client: Client<HttpsConnector<HttpConnector>>,
}

impl BinanceSource {
    /// Create a new Binance source provider
    pub fn new() -> Self {
        Self {
            http_client: Client::builder().build(HttpsConnector::new()),
        }
    }

    /// Get the average price for the given pair, approximating prices for
    /// pairs which don't natively exist on binance
    pub async fn approx_price_for_pair(&self, pair: &Pair) -> Result<Decimal, Error> {
        if let Ok(symbol_name) = SymbolName::try_from(pair) {
            return self.avg_price_for_symbol(symbol_name).await;
        }

        // Approximate prices by querying other currency pairs
        match pair {
            Pair(Currency::Luna, Currency::Krw) => {
                let (luna_btc, btc_bkrw) = tokio::join!(
                    self.avg_price_for_symbol(SymbolName::LunaBtc),
                    self.avg_price_for_symbol(SymbolName::BtcBkrw)
                );

                // Compute KRW by proxy using LUNA -> BTC -> KRW-ish
                Ok(luna_btc? * btc_bkrw?)
            }
            Pair(Currency::Luna, Currency::Usd) => {
                let (luna_busd, luna_usdt) = tokio::join!(
                    self.avg_price_for_symbol(SymbolName::LunaBusd),
                    self.avg_price_for_symbol(SymbolName::LunaUsdt)
                );

                // Give BUSD and USDT equal weight
                let avg = (luna_busd? + luna_usdt?) / Decimal::from(2);
                Ok(avg)
            }
            _ => fail!(ErrorKind::Currency, "unsupported Binance pair: {}", pair),
        }
    }

    /// `GET /api/v3/avgPrice` - get average price for Binance trading symbol
    pub async fn avg_price_for_symbol(&self, symbol_name: SymbolName) -> Result<Decimal, Error> {
        let uri = Uri::builder()
            .scheme("https")
            .authority(API_HOST)
            .path_and_query(format!("/api/v3/avgPrice?symbol={}", symbol_name).as_str())
            .build()
            .unwrap();

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

        let http_response = self.http_client.request(request).await?;
        let body = hyper::body::aggregate(http_response.into_body()).await?;

        let api_response: AvgPriceResponse = serde_json::from_reader(body.reader())?;
        Ok(api_response.price)
    }
}

impl Default for BinanceSource {
    fn default() -> Self {
        Self::new()
    }
}

/// Registered Binance trading pair symbols
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SymbolName {
    /// BTC/BKRW
    BtcBkrw,

    /// BTC/BUSD
    BtcBusd,

    /// BTC/GBP
    BtcGbp,

    /// BTC/EUR
    BtcEur,

    /// BTC/USDC
    BtcUsdc,

    /// BTC/USDT
    BtcUsdt,

    /// ETH/BTC
    EthBtc,

    /// ETH/BUSD
    EthBusd,

    /// ETH/EUR
    EthEur,

    /// ETH/GBP
    EthGbp,

    /// ETH/USDC
    EthUsdc,

    /// ETH/USDT
    EthUsdt,

    /// LUNA/BNB
    LunaBnb,

    /// LUNA/BTC
    LunaBtc,

    /// LUNA/BUSD
    LunaBusd,

    /// LUNA/USDT
    LunaUsdt,
}

impl Display for SymbolName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SymbolName::BtcBkrw => "BTCBKRW",
            SymbolName::BtcBusd => "BTCBUSD",
            SymbolName::BtcGbp => "BTCGBP",
            SymbolName::BtcEur => "BTCEUR",
            SymbolName::BtcUsdc => "BTCUSDC",
            SymbolName::BtcUsdt => "BTCUSDT",
            SymbolName::EthBtc => "ETHBTC",
            SymbolName::EthBusd => "ETHBUSD",
            SymbolName::EthEur => "ETHEUR",
            SymbolName::EthGbp => "ETHGBP",
            SymbolName::EthUsdc => "ETHUSDC",
            SymbolName::EthUsdt => "ETHUSDT",
            SymbolName::LunaBnb => "LUNABNB",
            SymbolName::LunaBtc => "LUNABTC",
            SymbolName::LunaBusd => "LUNABUSD",
            SymbolName::LunaUsdt => "LUNAUSDT",
        })
    }
}

impl FromStr for SymbolName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_ascii_uppercase().as_ref() {
            "BTCBKRW" => Ok(SymbolName::BtcBkrw),
            "BTCBUSD" => Ok(SymbolName::BtcBusd),
            "BTCGBP" => Ok(SymbolName::BtcGbp),
            "BTCEUR" => Ok(SymbolName::BtcEur),
            "BTCUSDC" => Ok(SymbolName::BtcUsdc),
            "BTCUSDT" => Ok(SymbolName::BtcUsdt),
            "ETHBTC" => Ok(SymbolName::EthBtc),
            "ETHBUSD" => Ok(SymbolName::EthBusd),
            "ETHEUR" => Ok(SymbolName::EthEur),
            "ETHGBP" => Ok(SymbolName::EthGbp),
            "ETHUSDC" => Ok(SymbolName::EthUsdc),
            "ETHUSDT" => Ok(SymbolName::EthUsdt),
            "LUNABNB" => Ok(SymbolName::LunaBnb),
            "LUNABTC" => Ok(SymbolName::LunaBtc),
            "LUNABUSD" => Ok(SymbolName::LunaBusd),
            "LUNAUSDT" => Ok(SymbolName::LunaUsdt),
            _ => fail!(ErrorKind::Currency, "unknown Binance symbol name: {}", s),
        }
    }
}

impl TryFrom<&Pair> for SymbolName {
    type Error = Error;

    fn try_from(pair: &Pair) -> Result<Self, Error> {
        // Strip slash from serialized pair
        pair.to_string()
            .chars()
            .filter(|&c| c != '/')
            .collect::<String>()
            .parse()
    }
}

/// Binance `/api/v3/avgPrice` response
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvgPriceResponse {
    /// Minutes the moving average is computed over
    pub mins: u32,

    /// Price
    pub price: Decimal,
}

#[cfg(test)]
mod tests {
    use super::BinanceSource;

    #[ignore]
    #[tokio::test]
    async fn avg_price_for_symbol() {
        let binance = BinanceSource::new();

        let luna_bnb = binance
            .avg_price_for_symbol("LUNABNB".parse().unwrap())
            .await
            .unwrap();

        dbg!(luna_bnb);

        let luna_btc = binance
            .avg_price_for_symbol("LUNABTC".parse().unwrap())
            .await
            .unwrap();

        dbg!(luna_btc);

        let luna_busd = binance
            .avg_price_for_symbol("LUNABUSD".parse().unwrap())
            .await
            .unwrap();

        dbg!(luna_busd);

        let luna_usdt = binance
            .avg_price_for_symbol("LUNAUSDT".parse().unwrap())
            .await
            .unwrap();

        dbg!(luna_usdt);
    }

    #[ignore]
    #[tokio::test]
    async fn approx_price_for_pair() {
        let binance = BinanceSource::new();

        let luna_btc = binance
            .approx_price_for_pair(&"LUNA/BTC".parse().unwrap())
            .await
            .unwrap();

        dbg!(luna_btc);

        let luna_krw = binance
            .approx_price_for_pair(&"LUNA/KRW".parse().unwrap())
            .await
            .unwrap();

        dbg!(luna_krw);

        let luna_usd = binance
            .approx_price_for_pair(&"LUNA/USD".parse().unwrap())
            .await
            .unwrap();

        dbg!(luna_usd);
    }
}
