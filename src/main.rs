mod config;
mod error;
mod server;
mod wallet;

#[macro_use]
extern crate log;

use std::env;

use crate::config::read_config;
use crate::server::create_server;
use crate::wallet::setup_wallet;

fn main() {
    env_logger::init();

    // Read configuration file
    let conf = match read_config() {
        Ok(config) => config,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    // Setup wallet
    let wallet = match setup_wallet(&conf) {
        Ok(wallet) => wallet,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    let host = conf.host.clone();

    // Start server
    let server = create_server(conf, wallet);

    server.listen(&host, &get_server_port().to_string());
}

/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080)
}
