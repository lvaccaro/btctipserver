use crate::error::Error;
use bdk::bitcoin::Network;
use ini::Ini;
use std::str::FromStr;

pub struct Config {
    pub datadir: String,
    pub descriptor: String,
    pub network: Network,
    pub wallet: String,
    pub electrum: String,
    pub host: String,
    pub port: String,
}

pub fn read_config() -> Result<Config, Error> {
    // load config from ini file
    let conf = Ini::load_from_file("config.ini").map_err(|_| Error::IniNotFound)?;

    let section_bdk = conf.section(Some("BDK")).ok_or(Error::MissingBDKSection)?;

    let datadir = section_bdk
        .get("datadir")
        .ok_or_else(|| Error::MissingParameter("datadir".to_string()))?
        .to_string();
    let descriptor = section_bdk
        .get("descriptor")
        .ok_or_else(|| Error::MissingParameter("descriptor".to_string()))?
        .to_string();
    let wallet = section_bdk
        .get("wallet")
        .ok_or_else(|| Error::MissingParameter("wallet".to_string()))?
        .to_string();
    let electrum = section_bdk
        .get("electrum")
        .ok_or_else(|| Error::MissingParameter("electrum".to_string()))?
        .to_string();
    let host = section_bdk.get("host").unwrap_or("0.0.0.0").to_string();
    let port = section_bdk.get("port").unwrap_or("8080").to_string();

    let network = Network::from_str(
        section_bdk
            .get("network")
            .ok_or_else(|| Error::MissingParameter("network".to_string()))?,
    )
    .map_err(|_| Error::InvalidNetwork)?;

    Ok(Config {
        datadir,
        descriptor,
        network,
        wallet,
        electrum,
        host,
        port,
    })
}
