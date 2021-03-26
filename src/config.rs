use bdk::bitcoin::Network;
use ini::Ini;
use std::env;
use std::path::Path;
use structopt;
use structopt::StructOpt;

/// Bitcoin tip server configuration options
///
/// All options can be set via command line arguments, environment variables or an ini file.
#[derive(Debug, StructOpt, Clone, PartialEq)]
#[structopt(name = "BTC Tip Server",
version = option_env ! ("CARGO_PKG_VERSION").unwrap_or("unknown"),
author = option_env ! ("CARGO_PKG_AUTHORS").unwrap_or(""),
about = "Top level options and command modes")]
pub struct ConfigOpts {
    /// Server host
    #[structopt(
        name = "HOST",
        env = "HOST",
        short = "h",
        long = "host",
        default_value = "0.0.0.0"
    )]
    pub host: String,
    /// Server port
    #[structopt(
        name = "PORT",
        env = "PORT",
        short = "p",
        long = "port",
        default_value = "8080"
    )]
    pub port: u16,
    /// Data Dir
    #[structopt(
        name = "DATADIR",
        env = "BDK_DATADIR",
        long = "datadir",
        default_value = ".bdk-bitcoin"
    )]
    pub data_dir: String,
    /// Bitcoin network
    #[structopt(
        name = "NETWORK",
        env = "NETWORK",
        short = "n",
        long = "network",
        default_value = "testnet",
        possible_values = &["bitcoin","testnet", "signet", "regtest"]
    )]
    pub network: Network,
    /// Wallet output descriptor, use public keys only
    #[structopt(
        name = "DESCRIPTOR",
        env = "DESCRIPTOR",
        short = "d",
        long = "descriptor"
    )]
    pub descriptor: String,
    /// Wallet name
    #[structopt(
        name = "WALLET",
        env = "WALLET",
        short = "w",
        long = "wallet",
        default_value = "btctipserver"
    )]
    pub wallet: String,
    #[structopt(flatten)]
    pub electrum_opts: ElectrumOpts,
}

impl ConfigOpts {
    /// Get config values from command line arg, env, or ini file in that order
    /// of precedence.
    pub fn from_ini_args<P>(filename: P) -> ConfigOpts
    where
        P: AsRef<Path>,
    {
        load_ini_to_env(filename);
        ConfigOpts::from_args()
    }
}

fn load_ini_to_env<P: AsRef<Path>>(filename: P) {
    // load config from ini file (if it exists) into process env
    if let Ok(conf) = Ini::load_from_file(filename) {
        if let Some(section_bdk) = conf.section(Some("BDK")) {
            for (k, v) in section_bdk.iter() {
                // if env var is not already set, set with ini value
                if env::var_os(k).is_none() {
                    env::set_var(k.to_uppercase(), v);
                }
            }
        }
    }
}

// This is a workaround for `structopt` issue #333, #391, #418; see https://github.com/TeXitoi/structopt/issues/333#issuecomment-712265332
#[cfg_attr(not(doc), allow(missing_docs))]
#[cfg_attr(
    doc,
    doc = r#"
Electrum options

Electrum blockchain client options.
"#
)]
#[derive(Debug, StructOpt, Clone, PartialEq)]
pub struct ElectrumOpts {
    /// Sets the SOCKS5 proxy for the Electrum client
    #[structopt(name = "PROXY_IP:PORT", env = "PROXY", long = "proxy")]
    pub proxy: Option<String>,
    /// Sets the SOCKS5 proxy retries for the Electrum client
    #[structopt(
        name = "PROXY_RETRIES",
        env = "RETRIES",
        long = "retries",
        default_value = "5"
    )]
    pub retries: u8,
    /// Sets the SOCKS5 proxy timeout for the Electrum client
    #[structopt(name = "PROXY_TIMEOUT", env = "TIMEOUT", long = "timeout")]
    pub timeout: Option<u8>,
    /// Sets the Electrum server to use
    #[structopt(
        name = "ELECTRUM_URL",
        env = "ELECTRUM",
        short = "s",
        long = "server",
        default_value = "ssl://electrum.blockstream.info:60002"
    )]
    pub electrum: String,
}

#[cfg(test)]
mod test {

    use super::{ConfigOpts, ElectrumOpts};
    use crate::config::load_ini_to_env;
    use bdk::bitcoin::Network;
    use std::ffi::OsString;
    use std::path::Path;
    use structopt;
    use structopt::StructOpt;

    /// Get config values from string iter, env, or ini file in that order
    /// of precedence.
    pub fn from_ini_iter<P, I>(filename: P, iter: I) -> ConfigOpts
    where
        P: AsRef<Path>,
        I: IntoIterator,
        I::Item: Into<OsString> + Clone,
    {
        load_ini_to_env(filename);
        ConfigOpts::from_iter(iter)
    }

    #[test]
    fn test_parse_config_opts() {
        let filename = "nonexistent_config.ini";
        let cli_args = vec!["btctipserver", "--network", "bitcoin",
                            "--descriptor", "wpkh(tpubEBr4i6yk5nf5DAaJpsi9N2pPYBeJ7fZ5Z9rmN4977iYLCGco1VyjB9tvvuvYtfZzjD5A8igzgw3HeWeeKFmanHYqksqZXYXGsw5zjnj7KM9/*)",
        ];

        let config_opts = from_ini_iter(filename, &cli_args);

        let expected_config_opts = ConfigOpts {
            host: "0.0.0.0".to_string(),
            port: 8080,
            data_dir: ".bdk-bitcoin".to_string(),
            network: Network::Bitcoin,
            descriptor: "wpkh(tpubEBr4i6yk5nf5DAaJpsi9N2pPYBeJ7fZ5Z9rmN4977iYLCGco1VyjB9tvvuvYtfZzjD5A8igzgw3HeWeeKFmanHYqksqZXYXGsw5zjnj7KM9/*)".parse().unwrap(),
            wallet: "main".to_string(),
            electrum_opts: ElectrumOpts {
                proxy: None,
                retries: 5,
                timeout: None,
                electrum: "ssl://electrum.blockstream.info:60002".to_string()
            }
        };

        assert_eq!(expected_config_opts, config_opts);
    }

    #[test]
    fn test_load_config_file() {
        let filename = "config_test.ini";
        let cli_args = vec!["btctipserver"];

        let config_opts = from_ini_iter(filename, cli_args);

        let expected_config_opts = ConfigOpts {
            host: "0.0.0.0".to_string(),
            port: 8080,
            data_dir: ".bdk-bitcoin".to_string(),
            network: Network::Bitcoin,
            descriptor: "wpkh(tpubEBr4i6yk5nf5DAaJpsi9N2pPYBeJ7fZ5Z9rmN4977iYLCGco1VyjB9tvvuvYtfZzjD5A8igzgw3HeWeeKFmanHYqksqZXYXGsw5zjnj7KM9/*)".parse().unwrap(),
            wallet: "test".to_string(),
            electrum_opts: ElectrumOpts {
                proxy: Some("127.0.0.1:9150".to_string()),
                retries: 5,
                timeout: Some(2),
                electrum: "ssl://electrum.blockstream.info:60003".to_string()
            }
        };

        assert_eq!(expected_config_opts, config_opts);
    }
}
