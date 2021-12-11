use std::fs;
use std::path::PathBuf;

use bdk::electrum_client::ListUnspentRes;

pub trait Wallet: Send {
    fn last_unused_address(&self) -> Result<String, bdk::Error>;

    fn is_my_address(&self, addr: &str) -> Result<bool, simple_server::Error>;

    fn check_address(
        &self,
        addr: &str,
        from_height: Option<usize>,
    ) -> Result<Vec<ListUnspentRes>, simple_server::Error>;

    fn network(&self) -> Result<String, bdk::Error>;
}

impl Wallet {
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
