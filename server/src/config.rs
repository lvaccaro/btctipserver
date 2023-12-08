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
    /// Server port
    #[structopt(
        name = "CONFIG",
        env = "CONFIG",
        short = "c",
        long = "config",
        default_value = "config.ini"
    )]
    pub config: String,
    #[structopt(subcommand)]
    pub cmd: Platforms,
}

#[derive(Debug, Clone, PartialEq, StructOpt)]
pub enum Platforms {
    #[structopt(name = "bitcoin", about = "use for bitcoin")]
    Bitcoin(btctipserver_bitcoin::config::BitcoinOpts),
    #[structopt(name = "liquid", about = "use for liquid")]
    Liquid(btctipserver_liquid::config::LiquidOpts),
    #[structopt(name = "clightning", about = "use for clightning with commando plugin")]
    CLightning(btctipserver_lightning::config::ClightningOpts),
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

#[cfg(test)]
mod test {

    use super::ConfigOpts;
    use crate::config::load_ini_to_env;
    use crate::config::Platforms;
    use btctipserver_bitcoin::bdk::bitcoin::Network;
    use btctipserver_bitcoin::config::{BitcoinOpts, ElectrumOpts};
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
            config: "config.ini".to_string(),
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
            config: "config.ini".to_string(),
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
