//! Exchange rate denominations

use crate::{
    config::AlphavantageConfig,
    currency::Currency,
    error::Error,
    sources::{
        alphavantage::AlphavantageSource, binance::BinanceSource, coinone::CoinoneSource,
        gdac::GdacSource, imf_sdr::ImfSDRSource, midpoint,
    },
    trading_pair::TradingPair,
};
use rust_decimal::Decimal;
use std::{
    convert::TryInto,
    fmt::{self, Display},
};

/// Denomination
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Denom {
    /// Korean Wan
    UKRW,

    /// Singaporean Dollar
    UMNT,

    /// IMF Special Drawing Rights
    USDR,

    /// US Dollars
    UUSD,
}

impl Denom {
    /// Get a slice of the [`Denom`] kinds
    pub fn kinds() -> &'static [Denom] {
        &[Denom::UKRW, Denom::UMNT, Denom::USDR, Denom::UUSD]
    }

    /// Get the code corresponding to a [`Denom`]
    pub fn as_str(self) -> &'static str {
        match self {
            Denom::UKRW => "ukrw",
            Denom::UMNT => "umnt",
            Denom::USDR => "usdr",
            Denom::UUSD => "uusd",
        }
    }

    /// Get the exchange rate for this [`Denom`]
    pub async fn get_exchange_rate(
        self,
        alphavantage_config: AlphavantageConfig,
    ) -> Result<stdtx::Decimal, Error> {
        match self {
            Denom::UKRW => {
                // Source: CoinOne
                let coinone_response = CoinoneSource::new()
                    .trading_pairs(&TradingPair(Currency::Luna, Currency::Krw))
                    .await?;
                // dbg!(&coinone_response);
                let coinone_midpoint = midpoint(&coinone_response)?;
                dbg!(&coinone_midpoint);

                // Source: GDAC
                let gdac_response = GdacSource::new()
                    .trading_pairs(&TradingPair(Currency::Luna, Currency::Krw))
                    .await?;
                // dbg!(&gdac_response);
                let gdac_midpoint = midpoint(&gdac_response)?;
                dbg!(&gdac_midpoint);

                // Source: Binance
                let binance_response = BinanceSource::new()
                    .approx_price_for_pair(&"LUNA/KRW".parse().unwrap())
                    .await
                    .unwrap();

                dbg!(&binance_response);

                //Midpoint avg for all sources
                let mut luna_krw =
                    Decimal::from((coinone_midpoint + gdac_midpoint + binance_response) / 3);
                dbg!(&luna_krw);

                dbg!(&luna_krw, luna_krw.scale());
                luna_krw.rescale(18);
                dbg!(&luna_krw, luna_krw.scale());
                Ok(luna_krw.try_into()?)
            }

            Denom::UMNT => {
                // Source: AlphaVantage
                let alphavantage_response = AlphavantageSource::new(alphavantage_config.apikey)
                    .trading_pairs(&TradingPair(
                        Currency::Krw,
                        Currency::Other("SGD".to_owned()),
                    ))
                    .await?;

                // Source: CoinOne
                let coinone_response = CoinoneSource::new()
                    .trading_pairs(&TradingPair(Currency::Luna, Currency::Krw))
                    .await?;
                // dbg!(&coinone_response);
                let coinone_midpoint = midpoint(&coinone_response)?;

                let mut luna_sgd = Decimal::from(
                    coinone_midpoint
                        * alphavantage_response
                            .realtime_currency_exchange_rate
                            .exchange_rate,
                );
                dbg!(luna_sgd);

                luna_sgd.rescale(18);

                Ok(luna_sgd.try_into()?)
            }

            Denom::UUSD => {
                // Source: Binance
                let binance_response = BinanceSource::new()
                    .approx_price_for_pair(&"LUNA/USD".parse().unwrap())
                    .await
                    .unwrap();

                let mut luna_usd: Decimal = binance_response.into();

                dbg!(luna_usd);

                luna_usd.rescale(18);

                Ok(luna_usd.try_into()?)
            }

            Denom::USDR => {
                //Source IMF_SDR
                let imf_sdr_response = ImfSDRSource::new()
                    .trading_pairs(&TradingPair(Currency::Krw, Currency::Sdr))
                    .await
                    .unwrap();

                // Source: CoinOne
                let coinone_response = CoinoneSource::new()
                    .trading_pairs(&TradingPair(Currency::Luna, Currency::Krw))
                    .await?;

                let coinone_midpoint = midpoint(&coinone_response)?;

                let mut luna_sdr = Decimal::from(coinone_midpoint * imf_sdr_response.price);

                dbg!(luna_sdr);

                luna_sdr.rescale(18);

                Ok(luna_sdr.try_into()?)
            }
        }
    }
}

impl Display for Denom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
