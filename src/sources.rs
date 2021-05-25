//! Data sources

pub mod alphavantage;
pub mod binance;
pub mod bithumb;
pub mod coinone;
pub mod currencylayer;
pub mod dunamu;
pub mod gdac;
pub mod gopax;
pub mod imf_sdr;

use self::{
    alphavantage::AlphavantageSource, binance::BinanceSource, bithumb::BithumbSource,
    coinone::CoinoneSource, currencylayer::CurrencylayerSource, dunamu::DunamuSource,
    gdac::GdacSource, gopax::GopaxSource, imf_sdr::ImfSdrSource,
};
use crate::{
    config::{source::AlphavantageConfig, DelphiConfig},
    Error, Price, PriceQuantity,
};
use rust_decimal::Decimal;

// TODO(shella): factor this into e.g. a common Tower service when we have 2+ oracles

/// Terra oracle sources
pub struct Sources {
    /// AlphaVantage
    /// <https://www.alphavantage.co/>
    pub alphavantage: AlphavantageSource,

    /// Binance
    /// <https://github.com/binance-exchange/binance-official-api-docs/blob/master/rest-api.md>
    pub binance: BinanceSource,

    /// CoinOne
    /// <https://coinone.co.kr/>
    pub coinone: CoinoneSource,

    /// Dunamu
    /// <https://dunamu.com/views/01_main.html>
    pub dunamu: DunamuSource,

    /// GDAC
    /// <https://www.gdac.com/?locale=en_us>
    pub gdac: GdacSource,

    /// GOPAX
    /// <https://www.gopax.co.id/API/>
    pub gopax: GopaxSource,

    /// IMF SDR
    /// <https://www.imf.org/>
    pub imf_sdr: ImfSdrSource,

    /// Bithumb
    /// <https://api.bithumb.com>
    pub bithumb: BithumbSource,

    /// Currencylayer
    /// <https://api.currencylayer.com>
    pub currencylayer: CurrencylayerSource,
}

impl Sources {
    /// Initialize sources from config
    pub fn new(config: &DelphiConfig) -> Result<Self, Error> {
        // TODO(tarcieri): support optionally enabling sources based on config
        let alphavantage = AlphavantageSource::new(
            &config
                .source
                .alphavantage
                .as_ref()
                .unwrap_or(&AlphavantageConfig {
                    apikey: "default key".to_string(),
                })
                .apikey,
            &config.https,
        )?;
        let binance = BinanceSource::new(&config.https)?;
        let coinone = CoinoneSource::new(&config.https)?;
        let gdac = GdacSource::new(&config.https)?;
        let dunamu = DunamuSource::new(&config.https)?;
        let gopax = GopaxSource::new(&config.https)?;
        let imf_sdr = ImfSdrSource::new(&config.https)?;
        let bithumb = BithumbSource::new(&config.https)?;
        let currencylayer = CurrencylayerSource::new(
            &config
                .source
                .currencylayer
                .as_ref()
                .expect("missing currencylayer config")
                .access_key,
            &config.https,
        )?;

        Ok(Sources {
            alphavantage,
            binance,
            coinone,
            dunamu,
            gdac,
            gopax,
            imf_sdr,
            bithumb,
            currencylayer,
        })
    }
}

///This trait allows writing generic functions over ask orderbook from multiple sources
pub trait AskBook {
    ///This function returns a vector of ask prices and volumes
    fn asks(&self) -> Result<Vec<PriceQuantity>, Error>;
}

///This trait allows writing generic functions over bid orderbook from multiple sources
pub trait BidBook {
    ///This function returns a vector of bid prices and volumes
    fn bids(&self) -> Result<Vec<PriceQuantity>, Error>;
}

/// Ask price weighted average
pub fn weighted_avg_ask<T: AskBook>(asks: &T) -> Result<Price, Error> {
    let asks = asks.asks()?;
    let mut price_sum_product = Decimal::from(0u8);
    let mut total = Decimal::from(0u8);
    for ask in asks {
        price_sum_product += *ask.price * ask.quantity;
        total += ask.quantity;
    }

    let weighted_avg = Price::new(price_sum_product / total)?;
    Ok(weighted_avg)
}

/// Bid price weighted average
pub fn weighted_avg_bid<T: BidBook>(bids: &T) -> Result<Price, Error> {
    let bids = bids.bids()?;
    let mut price_sum_product = Decimal::from(0u8);
    let mut total = Decimal::from(0u8);
    for bid in bids {
        price_sum_product += *bid.price * bid.quantity;
        total += bid.quantity;
    }

    let weighted_avg = Price::new(price_sum_product / total)?;
    Ok(weighted_avg)
}

/// Highest ask price
pub fn lowest_ask<T: AskBook>(asks: &T) -> Result<Price, Error> {
    let mut asks = asks.asks()?;
    asks.sort();
    Ok(asks.first().unwrap().price)
}

/// Lowest bid price
pub fn highest_bid<T: BidBook>(bids: &T) -> Result<Price, Error> {
    let mut bids = bids.bids()?;
    bids.sort();
    Ok(bids.last().unwrap().price)
}

/// Midpoint of highest ask and lowest bid price
pub fn midpoint<T: AskBook + BidBook>(book: &T) -> Result<Price, Error> {
    let lowest_ask = lowest_ask(book)?;
    let highest_bid = highest_bid(book)?;
    Ok((lowest_ask + highest_bid) / 2)
}
