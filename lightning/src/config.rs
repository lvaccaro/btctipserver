use structopt::StructOpt;

// This is a workaround for `structopt` issue #333, #391, #418; see https://github.com/TeXitoi/structopt/issues/333#issuecomment-712265332
#[cfg_attr(not(doc), allow(missing_docs))]
#[cfg_attr(
    doc,
    doc = r#"
Clightning options

Clightning commando wallet options.
"#
)]
#[derive(Debug, StructOpt, Clone, PartialEq)]
pub struct ClightningOpts {
    /// Lightning peer node id
    #[structopt(name = "NODEID", env = "NODEID", long = "nodeid")]
    pub nodeid: String,
    /// Lightning peer node ip or hostname
    #[structopt(
        name = "HOST",
        env = "HOST",
        long = "host",
        default_value = "127.0.0.1"
    )]
    pub host: String,
    /// Lightning rune auth from commando plugin
    #[structopt(name = "RUNE", env = "RUNE", long = "rune")]
    pub rune: String,
    /// Wallet output descriptor, use public keys only
    #[structopt(name = "PROXY", env = "PROXY", long = "proxy")]
    pub proxy: String,
}
