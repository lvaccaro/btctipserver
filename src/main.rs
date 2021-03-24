mod config;
mod error;
mod server;

#[macro_use]
extern crate log;
extern crate bdk;
extern crate bdk_macros;
extern crate env_logger;
extern crate ini;
extern crate serde_json;
extern crate simple_server;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::str;

use bdk::sled;
use bdk::Wallet;

use crate::config::{read_config, Config};
use crate::error::Error;
use crate::server::create_server;
use bdk::blockchain::{
    log_progress, AnyBlockchain, AnyBlockchainConfig, ConfigurableBlockchain,
    ElectrumBlockchainConfig,
};
use bdk::sled::Tree;

fn prepare_home_dir(datadir: &str) -> PathBuf {
    let mut dir = PathBuf::new();
    dir.push(&dirs_next::home_dir().unwrap());
    dir.push(datadir);

    if !dir.exists() {
        info!("Creating home directory {}", dir.as_path().display());
        fs::create_dir(&dir).unwrap();
    }

    dir.push("database.sled");
    dir
}

/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080)
}

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

fn setup_wallet(conf: &Config) -> Result<Wallet<AnyBlockchain, Tree>, Error> {
    // setup database
    let database = sled::open(prepare_home_dir(&conf.datadir).to_str().unwrap())?;
    let tree = database.open_tree(&conf.wallet)?;

    // setup electrum blockchain client
    let electrum_config = AnyBlockchainConfig::Electrum(ElectrumBlockchainConfig {
        url: conf.electrum.clone(),
        socks5: None,
        retry: 3,
        timeout: Some(2),
    });

    // create wallet shared by all requests
    let wallet = Wallet::new(
        &conf.descriptor,
        None,
        conf.network,
        tree,
        AnyBlockchain::from_config(&electrum_config)?,
    )?;
    wallet.sync(log_progress(), None)?;
    Ok(wallet)
}
