//! `start` subcommand

use crate::{application::APPLICATION, router::Router};
use abscissa_core::{prelude::*, Command, Options, Runnable};
use std::process;

/// `start` subcommand
#[derive(Command, Debug, Options)]
pub struct StartCmd {}

impl Runnable for StartCmd {
    /// Start the application.
    fn run(&self) {
        // Initialize router from the app's configuration
        let router = Router::init().unwrap_or_else(|e| {
            status_err!("{}", e);
            process::exit(1);
        });

        // Run the application
        abscissa_tokio::run(&APPLICATION, async { router.route().await }).unwrap_or_else(|e| {
            status_err!("executor exited with error: {}", e);
            process::exit(1);
        });
    }
}
