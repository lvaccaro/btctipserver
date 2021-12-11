use crate::config::BitcoinOpts;
use crate::error::Error;
use crate::server::gen_err;
use crate::wallet::Wallet;

use bdk::bitcoin::Address;
use bdk::blockchain::{
    log_progress, AnyBlockchain, AnyBlockchainConfig, ConfigurableBlockchain,
    ElectrumBlockchainConfig,
};
use bdk::electrum_client::{Client, ElectrumApi, ListUnspentRes};
use bdk::sled::{self, Tree};
use bdk::wallet::AddressIndex::LastUnused;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

pub struct BTCWallet {
    wallet: bdk::Wallet<AnyBlockchain, Tree>,
    client: Client,
}

impl BTCWallet {
    pub fn new(conf: &BitcoinOpts) -> Result<Self, Error> {
        // setup database
        let database = sled::open(Wallet::prepare_home_dir(&conf.data_dir).to_str().unwrap())?;
        let tree = database.open_tree(&conf.wallet)?;

        // setup electrum blockchain client
        let electrum_opts = conf.electrum_opts.clone();
        let electrum_config = AnyBlockchainConfig::Electrum(ElectrumBlockchainConfig {
            url: electrum_opts.electrum,
            socks5: electrum_opts.proxy,
            retry: electrum_opts.retries,
            timeout: electrum_opts.timeout,
        });

        // create wallet shared by all requests
        let wallet = bdk::Wallet::new(
            &conf.descriptor,
            None,
            conf.network,
            tree,
            AnyBlockchain::from_config(&electrum_config)?,
        )?;
        let client = Client::new(&conf.electrum_opts.electrum).unwrap();
        wallet.sync(log_progress(), None)?;
        Ok(BTCWallet { wallet, client })
    }

    pub fn prepare_home_dir(datadir: &str) -> PathBuf {
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
}

impl Wallet for BTCWallet {
    fn last_unused_address(&self) -> Result<String, bdk::Error> {
        self.wallet.sync(log_progress(), None)?;
        Ok(self.wallet.get_address(LastUnused)?.to_string())
    }

    fn is_my_address(&self, addr: &str) -> Result<bool, simple_server::Error> {
        let script = Address::from_str(addr)
            .map_err(|_| gen_err())?
            .script_pubkey();
        if self.wallet.is_mine(&script).map_err(|_| gen_err())? {
            return Ok(true);
        }
        self.wallet
            .sync(log_progress(), None)
            .map_err(|_| gen_err())?;
        self.wallet.is_mine(&script).map_err(|_| gen_err())
    }

    fn check_address(
        &self,
        addr: &str,
        from_height: Option<usize>,
    ) -> Result<Vec<ListUnspentRes>, simple_server::Error> {
        let monitor_script = Address::from_str(addr)
            .map_err(|_| gen_err())?
            .script_pubkey();

        let unspents = self
            .client
            .script_list_unspent(&monitor_script)
            .map_err(|_| gen_err())?;

        let array = unspents
            .into_iter()
            .filter(|x| x.height >= from_height.unwrap_or(0))
            .collect();

        Ok(array)
    }

    fn network(&self) -> Result<String, bdk::Error> {
        Ok(self.wallet.network().to_string())
    }
}
