mod btcwallet;
mod config;
mod clightning;
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
use crate::clightning::ClightningWallet;

use ini::Ini;
use std::env;
use structopt::StructOpt;

fn main() {
    env_logger::init();

    // Read configuration file to env if it exists, ignore otherwise
    let args: Vec<String> = env::args().collect();
    let config_file = match args.iter().position(|x| x == "--config") {
        Some(i) => &args[i + 1],
        None => "config.ini",
    };
    let _ = Ini::load_from_file(config_file).map(config::load_ini_to_env);

    // Read env and commandline args
    let conf: ConfigOpts = ConfigOpts::from_args();
    let server = match conf.cmd {
        Platforms::Bitcoin(opts) => {
            let wallet = BTCWallet::new(&opts).unwrap();
            server::create_server(wallet)
        }
        Platforms::Liquid(opts) => {
            let wallet = LiquidWallet::new(&opts).unwrap();
            server::create_server(wallet)
        }
        Platforms::CLightning(opts) => {
            let wallet = ClightningWallet::new(&opts).unwrap();
            server::create_server(wallet)
        }
    };

    // Start server
    let host = conf.host.clone();
    let port = conf.port.clone().to_string();
    server.listen(&host, &port);
}
