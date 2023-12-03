mod config;
mod html;
mod server;
mod wallet;

use btctipserver_bitcoin::BTCWallet;
use btctipserver_lightning::ClightningWallet;
use btctipserver_liquid::LiquidWallet;

use crate::config::{ConfigOpts, Platforms};
use crate::wallet::Wallet;

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
    let wallet = match conf.cmd {
        Platforms::Bitcoin(opts) => Wallet::BTCWallet(BTCWallet::new(&opts).unwrap()),
        Platforms::Liquid(opts) => Wallet::LiquidWallet(LiquidWallet::new(&opts).unwrap()),
        Platforms::CLightning(opts) => {
            Wallet::ClightningWallet(ClightningWallet::new(&opts).unwrap())
        }
    };

    // Start server
    let host = conf.host.clone();
    let port = conf.port.clone().to_string();
    let url = format!("{}:{}", host, port);
    server::run_server(url.as_str(), wallet)
}
