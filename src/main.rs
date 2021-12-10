mod btcwallet;
mod config;
mod error;
mod html;
mod server;

#[macro_use]
extern crate log;
extern crate http;

use crate::btcwallet::BTCWallet;
use crate::config::ConfigOpts;
use crate::server::create_server;
use ini::Ini;
use structopt::StructOpt;

fn main() {
    env_logger::init();

    // Read configuration file to env if it exists, ignore otherwise
    let _ = Ini::load_from_file("config.ini").map(config::load_ini_to_env);

    // Read env and commandline args
    let conf: ConfigOpts = ConfigOpts::from_args();

    // Setup wallet
    let btcwallet = match BTCWallet::new(&conf.bitcoin_opts) {
        Ok(btcwallet) => btcwallet,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    let host = conf.host.clone();
    let port = conf.port.clone().to_string();

    // Start server
    let server = create_server(conf.bitcoin_opts, btcwallet);

    server.listen(&host, &port);
}
