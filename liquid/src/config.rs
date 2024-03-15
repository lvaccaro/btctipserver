use std::str::FromStr;
use structopt::StructOpt;

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
        env = "LWK_DATADIR",
        long = "datadir",
        default_value = ".lwk"
    )]
    pub data_dir: String,
    /// Liquid network
    #[structopt(
        name = "NETWORK",
        env = "NETWORK",
        short = "n",
        long = "network",
        default_value = "liquidtestnet",
        possible_values = &["liquid","liquidtestnet","elements"]
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
    #[structopt(flatten)]
    pub electrum_opts: ElectrumOpts,
}
impl LiquidOpts {
    pub fn network(&self) -> lwk_wollet::ElementsNetwork {
        match self.network.as_str() {
            "liquid" => lwk_wollet::ElementsNetwork::Liquid,
            "liquidtestnet" => lwk_wollet::ElementsNetwork::LiquidTestnet,
            _ => {
                let policy_asset =
                    "5ac9f65c0efcc4775e0baec4ec03abdde22473cd3cf33c0419ca290e0751b225";
                let policy_asset =
                    lwk_wollet::elements::AssetId::from_str(policy_asset).expect("static");
                lwk_wollet::ElementsNetwork::ElementsRegtest { policy_asset }
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
