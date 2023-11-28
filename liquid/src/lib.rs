pub mod config;

pub extern crate edk;
extern crate structopt;

use crate::config::LiquidOpts;
use std::collections::HashMap;
use std::str::FromStr;
use edk::bdk::Error;
use edk::bdk::electrum_client::Client;

use edk::bdk::sled::{self, Tree};
use edk::miniscript::elements::secp256k1_zkp;
use edk::miniscript::elements::slip77::MasterBlindingKey;
use edk::miniscript::elements::Address;
use edk::miniscript::{Descriptor, DescriptorPublicKey};
use std::fs;
use std::path::PathBuf;

pub struct LiquidWallet {
    wallet: edk::Wallet<Tree>,
}

pub fn gen_err() -> Error {
    Error::Generic(format!("oh no!"))
}

impl LiquidWallet {

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

    pub fn new(opts: &LiquidOpts) -> Result<Self, Error> {
        // setup database
        let database = sled::open(Self::prepare_home_dir(&opts.data_dir).to_str().unwrap())?;
        let tree = database.open_tree(&opts.wallet)?;

        // setup electrum blockchain client
        let client = Client::new(&opts.electrum_opts.electrum).unwrap();

        // setup keys variables
        let descriptor = Descriptor::<DescriptorPublicKey>::from_str(&opts.descriptor).unwrap();
        let decoded: &[u8] = &hex::decode(&opts.master_blinding_key.as_str()).unwrap();
        let master_blinding_key =
            MasterBlindingKey(secp256k1_zkp::SecretKey::from_slice(decoded).unwrap());

        // create wallet shared by all requests
        let wallet = edk::Wallet::new(
            descriptor,
            master_blinding_key,
            tree,
            client,
            opts.network(),
        )
        .unwrap();
        Ok(LiquidWallet { wallet })
    }
}

impl LiquidWallet {
    pub fn last_unused_address(&mut self) -> Result<String, Error> {
        let address = self.wallet.get_new_address().map_err(|_| gen_err())?;
        Ok(address.to_string())
    }

    pub fn is_my_address(&mut self, addr: &str) -> Result<bool, Error> {
        let address = Address::from_str(addr).map_err(|_| gen_err())?;
        self.wallet.is_mine_address(&address).map_err(|_| gen_err())
    }

    pub fn balance_address(
        &mut self,
        addr: &str,
        _from_height: Option<usize>,
    ) -> Result<HashMap<String, u64>, Error> {
        let addr = Address::from_str(addr).map_err(|_| gen_err())?;
        let mut balances = HashMap::new();
        for unblind in self
            .wallet
            .balance_addresses(vec![addr])
            .map_err(|_| gen_err())?
            .unblinds
        {
            let tx_out = unblind.1;
            *balances.entry(tx_out.asset.to_string()).or_insert(0) += tx_out.value;
        }
        Ok(balances)
    }

    pub fn network(&mut self) -> Result<String, Error> {
        match self.wallet.network() {
            &edk::miniscript::elements::AddressParams::LIQUID => Ok("liquid".to_string()),
            _ => Ok("elements".to_string()),
        }
    }
}
