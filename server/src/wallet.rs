use btctipserver_bitcoin::BTCWallet;
use btctipserver_lightning::ClightningWallet;
use btctipserver_liquid::LiquidWallet;
use std::collections::HashMap;
use std::io;

pub fn gen_err() -> simple_server::Error {
    simple_server::Error::Io(io::Error::new(io::ErrorKind::Other, "oh no!"))
}

pub enum Wallet {
    BTCWallet(BTCWallet),
    ClightningWallet(ClightningWallet),
    LiquidWallet(LiquidWallet),
}
impl Wallet {

    pub fn last_unused_address(&mut self) -> Result<String, simple_server::Error> {
        match self {
            Wallet::BTCWallet(w) => { w.last_unused_address().map_err(|_| gen_err()) }
            Wallet::LiquidWallet(w) => { w.last_unused_address().map_err(|_| gen_err()) }
            Wallet::ClightningWallet(w) => { w.last_unused_address().map_err(|_| gen_err()) }
        }        
    }

    pub fn network(&mut self) -> Result<String, simple_server::Error> {
        match self {
            Wallet::BTCWallet(w) => { w.network().map_err(|_| gen_err()) }
            Wallet::LiquidWallet(w) => { w.network().map_err(|_| gen_err()) }
            Wallet::ClightningWallet(w) => { w.network().map_err(|_| gen_err()) }
        }        
    }

    pub fn is_my_address(&mut self, addr: &str) -> Result<bool, simple_server::Error> {
        match self {
            Wallet::BTCWallet(w) => { w.is_my_address(addr).map_err(|_| gen_err()) }
            Wallet::LiquidWallet(w) => { w.is_my_address(addr).map_err(|_| gen_err()) }
            Wallet::ClightningWallet(w) => { w.is_my_address(addr).map_err(|_| gen_err()) }
        }
    }

    pub fn balance_address(
        &mut self,
        addr: &str,
        _from_height: Option<usize>,
    ) -> Result<HashMap<String, u64>, simple_server::Error> {
        match self {
            Wallet::BTCWallet(w) => { w.balance_address(addr, _from_height).map_err(|_| gen_err()) }
            Wallet::LiquidWallet(w) => { w.balance_address(addr, _from_height).map_err(|_| gen_err()) }
            Wallet::ClightningWallet(w) => { w.balance_address(addr, _from_height).map_err(|_| gen_err()) }
        }
    }
}