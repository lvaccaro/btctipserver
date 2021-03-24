mod config;
mod error;

#[macro_use]
extern crate log;
extern crate bdk;
extern crate bdk_macros;
extern crate env_logger;
extern crate ini;
extern crate serde_json;
extern crate simple_server;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::str;
use std::str::FromStr;

use bdk::bitcoin::Address;
use bdk::sled;
use bdk::Wallet;

use bdk::electrum_client::{Client, ElectrumApi, ListUnspentRes};
use simple_server::{Method, Server, StatusCode};

use crate::config::read_config;
use bdk::blockchain::{
    log_progress, AnyBlockchain, AnyBlockchainConfig, ConfigurableBlockchain,
    ElectrumBlockchainConfig,
};
use bdk::sled::Tree;
use bdk::wallet::AddressIndex::LastUnused;
use std::sync::{Arc, Mutex};

fn prepare_home_dir(datadir: &str) -> PathBuf {
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

fn last_unused_address(wallet: &Wallet<AnyBlockchain, Tree>) -> Result<Address, bdk::Error> {
    wallet.sync(log_progress(), None)?;
    wallet.get_address(LastUnused)
}

fn check_address(
    client: &Client,
    addr: &str,
    from_height: Option<usize>,
) -> Result<Vec<ListUnspentRes>, bdk::Error> {
    let monitor_script = Address::from_str(addr).unwrap().script_pubkey();

    let unspents = client.script_list_unspent(&monitor_script).unwrap();

    let array = unspents
        .into_iter()
        .filter(|x| x.height >= from_height.unwrap_or(0))
        .collect();

    Ok(array)
}

fn html(electrum: &str, address: &str) -> Result<String, std::io::Error> {
    let client = Client::new(electrum).unwrap();
    let list = check_address(&client, &address, Option::from(0)).unwrap();

    let status = match list.last() {
        None => "No onchain tx found yet".to_string(),
        Some(unspent) => {
            let location = match unspent.height {
                0 => "in mempool".to_string(),
                _ => format!("at {}", unspent.height),
            };

            format!("Received {} sat {}", unspent.value, location)
        }
    };

    let template = fs::read_to_string("assets/index.html").unwrap();
    let link = format!("/bitcoin/?{}", address);
    let txt = template
        .replace("{address}", &address)
        .replace("{status}", &status)
        .replace("{refresh-link}", &link)
        .replace("{refresh-timeout}", "10");
    Ok(txt)
}

fn redirect(address: Address) -> Result<String, std::io::Error> {
    let link = format!("/bitcoin/?{}", address);
    let html = format!("<head><meta name='robots' content='noindex'><meta http-equiv=\"Refresh\" content=\"0; URL={}\"></head>", link);
    Ok(html)
}

/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080)
}

fn main() {
    env_logger::init();

    let conf = match read_config() {
        Ok(config) => config,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    // setup database
    let database = sled::open(prepare_home_dir(&conf.datadir).to_str().unwrap()).unwrap();
    let tree = database.open_tree(&conf.wallet).unwrap();

    // setup electrum blockchain client
    let electrum_config = AnyBlockchainConfig::Electrum(ElectrumBlockchainConfig {
        url: conf.electrum.clone(),
        socks5: None,
        retry: 3,
        timeout: Some(2),
    });

    // create wallet shared by all requests
    let wallet = Wallet::new(
        &conf.descriptor,
        None,
        conf.network,
        tree,
        AnyBlockchain::from_config(&electrum_config).unwrap(),
    )
    .unwrap();
    wallet.sync(log_progress(), None).unwrap();
    let wallet_mutex = Arc::new(Mutex::new(wallet));
    let host = conf.host.clone();

    let server = Server::new(move |request, mut response| {
        debug!("Request: {} {}", request.method(), request.uri());
        debug!("Body: {}", str::from_utf8(request.body()).unwrap());
        debug!("Headers:");
        for (key, value) in request.headers() {
            debug!("{}: {}", key, value.to_str().unwrap());
        }

        // unlock wallet mutex for this request
        let wallet = wallet_mutex.lock().unwrap();

        match (request.method(), request.uri().path()) {
            (&Method::GET, "/bitcoin/api/last_unused") => {
                let address = last_unused_address(&*wallet);
                return match address {
                    Ok(a) => {
                        info!("last unused addr {}", a.to_string());
                        let value = serde_json::json!({
                            "network": a.network.to_string(),
                            "address": a.to_string()
                        });
                        Ok(response.body(value.to_string().as_bytes().to_vec())?)
                    }
                    Err(e) => Ok(response.body(e.to_string().as_bytes().to_vec())?),
                };
            }
            (&Method::GET, "/bitcoin/api/check") => {
                // curl 127.0.0.1:7878/bitcoin/api/check?tb1qm4safqvzu28jvjz5juta7qutfaqst7nsfsumuz:0
                let mut query = request.uri().query().unwrap_or("").split(':');
                let addr = query.next().unwrap();
                let height = query.next().unwrap();
                let h: usize = height.parse::<usize>().unwrap();

                let client = Client::new(&conf.electrum).unwrap();
                let list = check_address(&client, &addr, Option::from(h));
                return match list {
                    Ok(list) => {
                        debug!("addr {} height {}", addr, h);
                        for item in list.iter() {
                            debug!("{} {}", item.value, item.height);
                            let _value = serde_json::json!({
                                "value": item.value,
                                "height": item.height,
                                "tx_hash": item.tx_hash,
                            });
                        }
                        Ok(response.body("".as_bytes().to_vec())?)
                    }
                    Err(e) => Ok(response.body(e.to_string().as_bytes().to_vec())?),
                };
            }
            (&Method::GET, "/bitcoin/") => {
                let address = request.uri().query().unwrap(); // TODO handle missing address
                return match html(&conf.electrum, address) {
                    Ok(txt) => Ok(response.body(txt.as_bytes().to_vec())?),
                    Err(e) => Ok(response.body(e.to_string().as_bytes().to_vec())?),
                };
            }
            (&Method::GET, "/bitcoin") => {
                let address = last_unused_address(&*wallet).unwrap();
                return match redirect(address) {
                    Ok(txt) => Ok(response.body(txt.as_bytes().to_vec())?),
                    Err(e) => Ok(response.body(e.to_string().as_bytes().to_vec())?),
                };
            }
            (&Method::GET, "/") => {
                let link = "/bitcoin";
                let redirect = format!("<head><meta name='robots' content='noindex'><meta http-equiv=\"Refresh\" content=\"0; URL={}\"></head>", link);
                Ok(response.body(redirect.as_bytes().to_vec())?)
            }
            (_, _) => {
                response.status(StatusCode::NOT_FOUND);
                Ok(response.body("<h1>404</h1><p>Not found!<p>".as_bytes().to_vec())?)
            }
        }
    });

    server.listen(&host, &get_server_port().to_string());
}
