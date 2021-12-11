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
use std::collections::HashMap;
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
}

impl Wallet for BTCWallet {
    fn last_unused_address(&self) -> Result<String, simple_server::Error> {
        self.wallet
            .sync(log_progress(), None)
            .map_err(|_| gen_err())?;
        Ok(self
            .wallet
            .get_address(LastUnused)
            .map_err(|_| gen_err())?
            .address
            .to_string())
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

    fn balance_address(
        &self,
        addr: &str,
        from_height: Option<usize>,
    ) -> Result<HashMap<String, u64>, simple_server::Error> {
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

    fn network(&self) -> Result<String, bdk::Error> {
        Ok(self.wallet.network().to_string())
    }
}
