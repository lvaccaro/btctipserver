pub mod config;
pub mod esplora;

pub extern crate lwk_wollet;
extern crate reqwest;
extern crate structopt;

use crate::config::LiquidOpts;
use lwk_wollet::elements::Address;
use lwk_wollet::{
    full_scan_with_electrum_client, ElectrumClient, ElectrumUrl, Error, FsPersister, Wollet,
};

use esplora::EsploraRepository;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

pub struct LiquidWallet {
    network: lwk_wollet::ElementsNetwork,
    wollet: Wollet,
    electrum: ElectrumClient,
    esplora: EsploraRepository,
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
        dir
    }

    pub fn new(opts: &LiquidOpts) -> Result<Self, Error> {
        // setup database
        let datadir = Self::prepare_home_dir(&opts.data_dir);
        //let tree = database.open_tree(&opts.wallet)?;

        // setup keys variables
        //let descriptor: WolletDescriptor = opts.descriptor.parse()?;
        let mut wollet = Wollet::new(
            opts.network(),
            FsPersister::new(&datadir).unwrap(),
            opts.descriptor.as_str(),
        )
        .unwrap();

        // setup electrum blockchain client
        let electrum_url = ElectrumUrl::new(&opts.electrum_opts.electrum, true, true);
        let mut electrum_client = ElectrumClient::new(&electrum_url)?;

        full_scan_with_electrum_client(&mut wollet, &mut electrum_client)?;

        Ok(LiquidWallet {
            network: opts.network(),
            wollet,
            electrum: electrum_client,
            esplora: EsploraRepository {
                assets: HashMap::new(),
            },
        })
    }
}

impl LiquidWallet {
    pub fn last_unused_address(&mut self) -> Result<String, Error> {
        let address = self.wollet.address(None).map_err(|_| gen_err())?;
        Ok(address.address().to_string())
    }

    pub fn is_my_address(&mut self, addr: &str) -> Result<bool, Error> {
        let _address = Address::from_str(addr).map_err(|_| gen_err())?;
        //self.wollet.is_mine_address(&address).map_err(|_| gen_err())
        Ok(true)
    }

    pub fn balance_address(
        &mut self,
        addr: &str,
        _from_height: Option<usize>,
    ) -> Result<HashMap<String, String>, Error> {
        full_scan_with_electrum_client(&mut self.wollet, &mut self.electrum)?;
        let script_pubkey = Address::from_str(addr).unwrap().script_pubkey();
        let tx_outs = self
            .wollet
            .utxos()
            .unwrap()
            .into_iter()
            .filter(|x| x.script_pubkey == script_pubkey);
        let mut balance = HashMap::new();
        for out in tx_outs {
            let asset_id = out.unblinded.asset;
            let precision = self.esplora.get(asset_id).map_or(0, |x| x.precision);
            let decimal = (10 as f64).powf(precision as f64);
            let value = out.unblinded.value as f64 / decimal;
            balance
                .entry(out.unblinded.asset)
                .and_modify(|x| *x += value)
                .or_insert(value);
        }
        let mut res = HashMap::new();
        for b in balance {
            let ticker = self.esplora.get(b.0).map_or(b.0.to_string(), |x| x.ticker);
            res.insert(ticker, format!("{}", b.1));
        }
        println!("balance {:?}", res);
        Ok(res)
    }

    pub fn network(&mut self) -> Result<String, Error> {
        match self.network {
            lwk_wollet::ElementsNetwork::Liquid => Ok("Liquid".to_string()),
            lwk_wollet::ElementsNetwork::LiquidTestnet => Ok("Liquid Testnet".to_string()),
            _ => Ok("Elements".to_string()),
        }
    }
}
