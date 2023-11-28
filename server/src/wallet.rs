use btctipserver_bitcoin::BTCWallet;
use btctipserver_lightning::ClightningWallet;
use btctipserver_liquid::LiquidWallet;
use std::collections::HashMap;

/// Errors that can be thrown by the [`Wallet`](crate::wallet::Wallet)
#[derive(Debug)]
pub enum Error {
    Generic(String)
}

pub fn gen_err() -> Error {
    Error::Generic(format!("oh no!"))
}

pub enum Wallet {
    BTCWallet(BTCWallet),
    ClightningWallet(ClightningWallet),
    LiquidWallet(LiquidWallet),
}
impl Wallet {

    pub fn last_unused_address(&mut self) -> Result<String, Error> {
        match self {
            Wallet::BTCWallet(w) => { w.last_unused_address().map_err(|_| gen_err()) }
            Wallet::LiquidWallet(w) => { w.last_unused_address().map_err(|_| gen_err()) }
            Wallet::ClightningWallet(w) => { w.last_unused_address().map_err(|_| gen_err()) }
        }        
    }

    pub fn network(&mut self) -> Result<String, Error> {
        match self {
            Wallet::BTCWallet(w) => { w.network().map_err(|_| gen_err()) }
            Wallet::LiquidWallet(w) => { w.network().map_err(|_| gen_err()) }
            Wallet::ClightningWallet(w) => { w.network().map_err(|_| gen_err()) }
        }        
    }

    pub fn is_my_address(&mut self, addr: &str) -> Result<bool, Error> {
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
    ) -> Result<HashMap<String, u64>, Error> {
        match self {
            Wallet::BTCWallet(w) => { w.balance_address(addr, _from_height).map_err(|_| gen_err()) }
            Wallet::LiquidWallet(w) => { w.balance_address(addr, _from_height).map_err(|_| gen_err()) }
            Wallet::ClightningWallet(w) => { w.balance_address(addr, _from_height).map_err(|_| gen_err()) }
        }
    }
}