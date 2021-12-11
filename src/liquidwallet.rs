use crate::config::LiquidOpts;
use crate::error::Error;
use crate::wallet::Wallet;

use std::str::FromStr;

use edk::bdk::electrum_client::{Client};

use bdk::sled::{self, Tree};
use edk::miniscript::elements::secp256k1_zkp;
use edk::miniscript::elements::slip77::MasterBlindingKey;
use edk::miniscript::{Descriptor, DescriptorPublicKey};
use hex;

pub struct LiquidWallet {
    wallet: edk::Wallet<Tree>,
}

impl LiquidWallet {
    pub fn new(opts: &LiquidOpts) -> Result<Self, Error> {
        // setup database
        let database = sled::open(Wallet::prepare_home_dir(&opts.data_dir).to_str().unwrap())?;
        let tree = database.open_tree(&opts.wallet)?;

        // setup electrum blockchain client
        let electrum_opts = opts.electrum_opts.clone();
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

impl Wallet for LiquidWallet {
    fn last_unused_address(&self) -> Result<String, bdk::Error> {
        Ok("".to_string())
    }

    fn is_my_address(&self, addr: &str) -> Result<bool, simple_server::Error> {
        Ok(true)
    }

    fn check_address(
        &self,
        addr: &str,
        from_height: Option<usize>,
    ) -> Result<Vec<bdk::electrum_client::ListUnspentRes>, simple_server::Error> {
        Ok(vec![])
    }

    fn network(&self) -> Result<String, bdk::Error> {
        Ok("liquid".to_string())
    }
}
