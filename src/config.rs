use bdk::bitcoin::Network;
use ini::Ini;
use std::env;
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
    #[structopt(subcommand)]
    pub cmd: Platforms,
}

#[derive(Debug, Clone, PartialEq, StructOpt)]
pub enum Platforms {
    #[structopt(name = "bitcoin", about = "use for bitcoin")]
    Bitcoin(BitcoinOpts),
    #[structopt(name = "liquid", about = "use for liquid")]
    Liquid(LiquidOpts),
}

pub(crate) fn load_ini_to_env(ini: Ini) {
    // load config from ini file (if it exists) into process env
    if let Some(section_bdk) = ini.section(None::<String>) {
        for (k, v) in section_bdk.iter() {
            // if env var is not already set, set with ini value
            if env::var_os(k).is_none() {
                env::set_var(k.to_uppercase(), v);
            }
        }
    }
}

// This is a workaround for `structopt` issue #333, #391, #418; see https://github.com/TeXitoi/structopt/issues/333#issuecomment-712265332
#[cfg_attr(not(doc), allow(missing_docs))]
#[cfg_attr(
    doc,
    doc = r#"
Bitcoin options

Bitcoin wallet options.
"#
)]
#[derive(Debug, StructOpt, Clone, PartialEq)]
pub struct BitcoinOpts {
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

// This is a workaround for `structopt` issue #333, #391, #418; see https://github.com/TeXitoi/structopt/issues/333#issuecomment-712265332
#[cfg_attr(not(doc), allow(missing_docs))]
#[cfg_attr(
    doc,
    doc = r#"
Liquid options

Liquid wallet options.
"#
)]
#[derive(Debug, StructOpt, Clone, PartialEq)]
pub struct LiquidOpts {
    /// Data Dir
    #[structopt(
        name = "DATADIR",
        env = "EDK_DATADIR",
        long = "datadir",
        default_value = ".edk-bitcoin"
    )]
    pub data_dir: String,
    /// Liquid network
    #[structopt(
        name = "NETWORK",
        env = "NETWORK",
        short = "n",
        long = "network",
        default_value = "elements",
        possible_values = &["liquid","elements"]
    )]
    pub network: String,
    /// Wallet output descriptor, use public keys only
    #[structopt(
        name = "DESCRIPTOR",
        env = "DESCRIPTOR",
        short = "d",
        long = "descriptor"
    )]
    pub descriptor: String,
    /// Wallet output descriptor, use public keys only
    #[structopt(
        name = "MASTER_BLINDING_KEY",
        env = "MASTER_BLINDING_KEY",
        short = "b",
        long = "master_blinding_key"
    )]
    pub master_blinding_key: String,
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

impl LiquidOpts {
    pub fn network(&self) -> &'static edk::miniscript::elements::AddressParams {
        match self.network.as_str() {
            "liquid" => &edk::miniscript::elements::AddressParams::LIQUID,
            _ => &edk::miniscript::elements::AddressParams::ELEMENTS,
        }
    }
}

#[cfg(test)]
mod test {

    use super::{BitcoinOpts, ConfigOpts, ElectrumOpts};
    use crate::config::load_ini_to_env;
    use crate::config::Platforms;
    use bdk::bitcoin::Network;
    use ini::Ini;
    use structopt;
    use structopt::StructOpt;

    #[test]
    fn test_config_from_args() {
        let cli_args = vec!["btctipserver", "--network", "bitcoin",
                            "--descriptor", "wpkh(tpubEBr4i6yk5nf5DAaJpsi9N2pPYBeJ7fZ5Z9rmN4977iYLCGco1VyjB9tvvuvYtfZzjD5A8igzgw3HeWeeKFmanHYqksqZXYXGsw5zjnj7KM9/*)",
        ];

        let config_opts: ConfigOpts = ConfigOpts::from_iter(&cli_args);

        let expected_config_opts = ConfigOpts {
            host: "0.0.0.0".to_string(),
            port: 8080,
            cmd: Platforms::Bitcoin( BitcoinOpts {
                data_dir: ".bdk-bitcoin".to_string(),
                network: Network::Bitcoin,
                descriptor: "wpkh(tpubEBr4i6yk5nf5DAaJpsi9N2pPYBeJ7fZ5Z9rmN4977iYLCGco1VyjB9tvvuvYtfZzjD5A8igzgw3HeWeeKFmanHYqksqZXYXGsw5zjnj7KM9/*)".parse().unwrap(),
                wallet: "btctipserver".to_string(),
                electrum_opts: ElectrumOpts {
                    proxy: None,
                    retries: 5,
                    timeout: None,
                    electrum: "ssl://electrum.blockstream.info:60002".to_string()
                }
            })
        };

        assert_eq!(expected_config_opts, config_opts);
    }

    #[test]
    fn test_config_from_ini_env() {
        let config = r#"
            [BDK]
            datadir = .bdk-bitcoin
            network = bitcoin
            wallet = test
            descriptor = 'wpkh(tpubEBr4i6yk5nf5DAaJpsi9N2pPYBeJ7fZ5Z9rmN4977iYLCGco1VyjB9tvvuvYtfZzjD5A8igzgw3HeWeeKFmanHYqksqZXYXGsw5zjnj7KM9/*)'
            electrum = 'ssl://electrum.blockstream.info:60003'
            proxy = 127.0.0.1:9150
            retries = 5
            timeout = 2
        "#;

        let ini = Ini::load_from_str(config).unwrap();
        load_ini_to_env(ini);

        let cli_args = vec!["btctipserver"];

        let config_opts: ConfigOpts = ConfigOpts::from_iter(&cli_args);

        let expected_config_opts = ConfigOpts {
            host: "0.0.0.0".to_string(),
            port: 8080,
            cmd: Platforms::Bitcoin( BitcoinOpts {
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
            }),
        };

        assert_eq!(expected_config_opts, config_opts);
    }
}
