pub mod config;

pub extern crate bdk;
extern crate log;
extern crate hex;
extern crate http;
extern crate ini;
extern crate structopt;

use bdk::bitcoin::Address;
use bdk::blockchain::{
    log_progress, AnyBlockchain, AnyBlockchainConfig, ConfigurableBlockchain,
    ElectrumBlockchainConfig,
};
use bdk::electrum_client::{Client, ElectrumApi, ListUnspentRes};
use bdk::sled::{self, Tree};
use bdk::wallet::AddressIndex::LastUnused;
use std::collections::HashMap;
use std::str::FromStr;
use config::BitcoinOpts;
use std::fs;
use std::path::PathBuf;

pub fn gen_err() -> bdk::Error {
    bdk::Error::Generic(format!("oh no!"))
}

pub struct BTCWallet {
    wallet: bdk::Wallet<AnyBlockchain, Tree>,
    client: Client,
}

impl BTCWallet {

    pub fn prepare_home_dir(datadir: &str) -> PathBuf {
        let mut dir = PathBuf::new();
        dir.push(&dirs_next::home_dir().unwrap());
        dir.push(datadir);

        if !dir.exists() {
            //info!("Creating home directory {}", dir.as_path().display());
            fs::create_dir(&dir).unwrap();
        }

        dir.push("database.sled");
        dir
    }

    pub fn new(conf: &BitcoinOpts) -> Result<Self, bdk::Error> {
        // setup database
        env_logger::init();
        let database = sled::open(Self::prepare_home_dir(&conf.data_dir).to_str().unwrap())?;
        let tree = database.open_tree(&conf.wallet)?;

        // setup electrum blockchain client
        let electrum_opts = conf.electrum_opts.clone();
        let electrum_config = AnyBlockchainConfig::Electrum(ElectrumBlockchainConfig {
            url: electrum_opts.electrum,
            socks5: electrum_opts.proxy,
            retry: electrum_opts.retries,
            timeout: electrum_opts.timeout,
            stop_gap: 20,
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

    fn check_address(
        &self,
        addr: &str,
        from_height: Option<usize>,
    ) -> Result<Vec<ListUnspentRes>, bdk::Error> {
        let monitor_script = Address::from_str(addr).map_err(|_| gen_err())?
            .script_pubkey();

        let unspents = self
            .client
            .script_list_unspent(&monitor_script)?;

        let array = unspents
            .into_iter()
            .filter(|x| x.height >= from_height.unwrap_or(0))
            .collect();

        Ok(array)
    }
}

impl BTCWallet {
    pub fn last_unused_address(&mut self) -> Result<String, bdk::Error> {
        let _ = self.wallet
            .sync(log_progress(), None);
        Ok(self
            .wallet
            .get_address(LastUnused)?
            .address
            .to_string())
    }

    pub fn is_my_address(&mut self, addr: &str) -> Result<bool, bdk::Error> {
        let script = Address::from_str(addr).map_err(|_| gen_err())?
            .script_pubkey();
        if self.wallet.is_mine(&script)? {
            return Ok(true);
        }
        let _ = self.wallet
            .sync(log_progress(), None);
        self.wallet.is_mine(&script)
    }

    pub fn balance_address(
        &mut self,
        addr: &str,
        from_height: Option<usize>,
    ) -> Result<HashMap<String, u64>, bdk::Error> {
        let list = self.check_address(addr, from_height)?;
        let mut balances = HashMap::new();

        let amount = match list.last() {
            None => 0,
            Some(unspent) => {
                if unspent.height < from_height.unwrap_or(0) {
                    0
                } else {
                    unspent.value
                }
            }
        };
        balances.insert("btc".to_string(), amount);
        Ok(balances)
    }

    pub fn network(&mut self) -> Result<String, bdk::Error> {
        Ok(self.wallet.network().to_string())
    }
}
