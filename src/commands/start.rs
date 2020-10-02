//! `start` subcommand

use crate::{application::APPLICATION, networks::terra};
use abscissa_core::{prelude::*, Command, Options, Runnable};
use std::process;
use warp::Filter;

/// Feeder address
pub const FEEDER_ADDR: &str = "terra1t9et8wjeh8d0ewf4lldchterxsmhpcgg5auy47";

/// Validator address
pub const VALIDATOR_ADDR: &str = "terravaloper1grgelyng2v6v3t8z87wu3sxgt9m5s03x2mfyu7";

/// `start` subcommand
#[derive(Command, Debug, Options)]
pub struct StartCmd {}

impl Runnable for StartCmd {
    /// Start the application.
    fn run(&self) {
        abscissa_tokio::run(&APPLICATION, async {
            // TODO(tarcieri): configure listen addr/port from config file
            let listen_addr = [127, 0, 0, 1];
            let listen_port = 23456;

            // TODO(tarcieri): load feeder/validator from config file
            let feeder = stdtx::Address::from_bech32(FEEDER_ADDR).unwrap().1;
            let validator = stdtx::Address::from_bech32(VALIDATOR_ADDR).unwrap().1;

            let terra_oracle = terra::ExchangeRateOracle::new(feeder, validator);
            let terra_oracle_filter = warp::any().map(move || terra_oracle.clone());

            let app = warp::post()
                .and(warp::path("oracles"))
                .and(warp::path("terra"))
                .and(warp::path::end())
                .and(terra_oracle_filter.clone())
                .and_then(terra::ExchangeRateOracle::handle_request);

            warp::serve(app).run((listen_addr, listen_port)).await;
        })
        .unwrap_or_else(|e| {
            status_err!("executor exited with error: {}", e);
            process::exit(1);
        });
    }
}
