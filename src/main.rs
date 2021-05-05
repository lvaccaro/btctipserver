mod bip21;
mod config;
mod error;
mod html;
mod server;
mod wallet;

#[macro_use]
extern crate log;

use crate::config::ConfigOpts;
use crate::server::create_server;
use crate::wallet::setup_wallet;
use ini::Ini;
use structopt::StructOpt;

fn main() {
    env_logger::init();

    // Read configuration file to env if it exists, ignore otherwise
    let _ = Ini::load_from_file("config.ini").map(config::load_ini_to_env);

    // Read env and commandline args
    let conf: ConfigOpts = ConfigOpts::from_args();

    // Setup wallet
    let wallet = match setup_wallet(&conf) {
        Ok(wallet) => wallet,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    let host = conf.host.clone();
    let port = conf.port.clone().to_string();

    // Start server
    let server = create_server(conf, wallet);

    server.listen(&host, &port);
}
