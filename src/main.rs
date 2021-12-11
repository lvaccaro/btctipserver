mod btcwallet;
mod config;
mod error;
mod html;
mod liquidwallet;
mod server;
mod wallet;

#[macro_use]
extern crate log;
extern crate hex;
extern crate http;

use crate::btcwallet::BTCWallet;
use crate::config::{ConfigOpts, Platforms};
use crate::liquidwallet::LiquidWallet;

use crate::html::not_found;
use crate::wallet::Wallet;

use ini::Ini;
use std::sync::{Arc, Mutex, MutexGuard};
use structopt::StructOpt;

use http::Uri;
use simple_server::{Method, Request, Response, ResponseBuilder, Server, StatusCode};

fn main() {
    env_logger::init();

    // Read configuration file to env if it exists, ignore otherwise
    let _ = Ini::load_from_file("config.ini").map(config::load_ini_to_env);

    // Read env and commandline args
    let conf: ConfigOpts = ConfigOpts::from_args();
    let server = match conf.sub {
        Platforms::Bitcoin(opts) => {
            let wallet = BTCWallet::new(&opts).unwrap();
            server::create_server(wallet)
        }
        Platforms::Liquid(opts) => {
            let wallet = LiquidWallet::new(&opts).unwrap();
            server::create_server(wallet)
        }
    };

    // Start server
    let host = conf.host.clone();
    let port = conf.port.clone().to_string();
    server.listen(&host, &port);
}
