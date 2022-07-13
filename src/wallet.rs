use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub trait Wallet: Send {
    fn last_unused_address(&mut self) -> Result<String, simple_server::Error>;

    fn is_my_address(&mut self, addr: &str) -> Result<bool, simple_server::Error>;

    fn balance_address(
        &mut self,
        addr: &str,
        from_height: Option<usize>,
    ) -> Result<HashMap<String, u64>, simple_server::Error>;

    fn network(&mut self) -> Result<String, bdk::Error>;
}

impl dyn Wallet {
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
