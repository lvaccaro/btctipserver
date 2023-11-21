use bdk::bitcoin::Network;
use structopt::StructOpt;

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