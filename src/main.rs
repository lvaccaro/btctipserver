mod config;
mod error;
mod server;
mod wallet;

#[macro_use]
extern crate log;

use crate::config::ConfigOpts;
use crate::server::create_server;
use crate::wallet::setup_wallet;

fn main() {
    env_logger::init();

    // Read configuration file
    let conf = ConfigOpts::from_ini_args("config.ini");

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
