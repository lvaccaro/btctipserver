mod config;
mod error;
mod html;
mod server;
mod btcwallet;

#[macro_use]
extern crate log;

use crate::config::ConfigOpts;
use crate::server::create_server;
use crate::btcwallet::BTCWallet;
use ini::Ini;
use structopt::StructOpt;

fn main() {
    env_logger::init();

    // Read configuration file to env if it exists, ignore otherwise
    let _ = Ini::load_from_file("config.ini").map(config::load_ini_to_env);

    // Read env and commandline args
    let conf: ConfigOpts = ConfigOpts::from_args();

    // Setup wallet
    let btcwallet = match BTCWallet::new(&conf) {
        Ok(btcwallet) => btcwallet,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    let host = conf.host.clone();
    let port = conf.port.clone().to_string();

    // Start server
    let server = create_server(conf, btcwallet);

    server.listen(&host, &port);
}
