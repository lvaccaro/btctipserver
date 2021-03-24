use crate::config::Config;
use bdk::bitcoin::Address;
use bdk::blockchain::{log_progress, AnyBlockchain};
use bdk::electrum_client::{Client, ElectrumApi, ListUnspentRes};
use bdk::sled::Tree;
use bdk::wallet::AddressIndex::LastUnused;
use bdk::Wallet;
use simple_server::{Method, Server, StatusCode};
use std::fs;
use std::str::{from_utf8, FromStr};
use std::sync::{Arc, Mutex};

pub fn create_server(conf: Config, wallet: Wallet<AnyBlockchain, Tree>) -> Server {
    let wallet_mutex = Arc::new(Mutex::new(wallet));
    Server::new(move |request, mut response| {
        debug!("Request: {} {}", request.method(), request.uri());
        debug!("Body: {}", from_utf8(request.body()).unwrap());
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
    })
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
