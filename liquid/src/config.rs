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
impl LiquidOpts {
    pub fn network(&self) -> &'static edk::miniscript::elements::AddressParams {
        match self.network.as_str() {
            "liquid" => &edk::miniscript::elements::AddressParams::LIQUID,
            _ => &edk::miniscript::elements::AddressParams::ELEMENTS,
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
