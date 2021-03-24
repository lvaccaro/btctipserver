use crate::config::Config;
use crate::error::Error;
use bdk::blockchain::{
    log_progress, AnyBlockchain, AnyBlockchainConfig, ConfigurableBlockchain,
    ElectrumBlockchainConfig,
};
use bdk::sled::{self, Tree};
use bdk::Wallet;
use std::fs;
use std::path::PathBuf;

pub fn setup_wallet(conf: &Config) -> Result<Wallet<AnyBlockchain, Tree>, Error> {
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
