//! `start` subcommand

use crate::{application::APPLICATION, networks::terra};
use abscissa_core::{prelude::*, Command, Options, Runnable};
use std::process;
use warp::Filter;

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

            let terra_oracle = terra::ExchangeRateOracle::new();
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
