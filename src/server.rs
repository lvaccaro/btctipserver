use crate::config::ConfigOpts;
use bdk::bitcoin::Address;
use bdk::blockchain::{log_progress, AnyBlockchain};
use bdk::electrum_client::{Client, ElectrumApi, ListUnspentRes};
use bdk::sled::Tree;
use bdk::wallet::AddressIndex::LastUnused;
use bdk::Wallet;
use simple_server::{Method, Server, StatusCode};
use std::str::{from_utf8, FromStr};
use std::sync::{Arc, Mutex};

use crate::html;
use crate::html::not_found;
use std::io;

/// Returns a generic simple_server::Error, used to catch errors to prevent tearing
/// down the server with a simple request, should be removed in favor of specific errors
fn gen_err() -> simple_server::Error {
    simple_server::Error::Io(io::Error::new(io::ErrorKind::Other, "oh no!"))
}

pub fn create_server(conf: ConfigOpts, wallet: Wallet<AnyBlockchain, Tree>) -> Server {
    let wallet_mutex = Arc::new(Mutex::new(wallet));

    Server::new(move |request, mut response| {
        debug!("Request: {} {}", request.method(), request.uri());
        debug!(
            "Body: {}",
            from_utf8(request.body()).map_err(|_| gen_err())?
        );
        debug!("Headers:");
        for (key, value) in request.headers() {
            debug!("{}: {}", key, value.to_str().unwrap_or("can't map to str"));
        }

        // unlock wallet mutex for this request
        let wallet = wallet_mutex.lock().map_err(|_| gen_err())?;

        match (request.method(), request.uri().path()) {
            (&Method::GET, "/bitcoin/api/last_unused_qr.bmp") => {
                let address = last_unused_address(&*wallet);
                match address {
                    Ok(addr) => {
                        info!("last unused addr {}", addr.to_string());
                        let qr = html::create_bmp_qr(addr.to_string().as_str())
                            .map_err(|_| gen_err())?;
                        response.header("Content-type", "image/bmp");
                        Ok(response.body(qr)?)
                    }
                    Err(e) => Ok(response.body(e.to_string().as_bytes().to_vec())?),
                }
            }
            (&Method::GET, "/bitcoin/api/last_unused") => {
                let address = last_unused_address(&*wallet);
                match address {
                    Ok(a) => {
                        info!("last unused addr {}", a.to_string());
                        let value = serde_json::json!({
                            "network": a.network.to_string(),
                            "address": a.to_string()
                        });
                        Ok(response.body(value.to_string().as_bytes().to_vec())?)
                    }
                    Err(e) => Ok(response.body(e.to_string().as_bytes().to_vec())?),
                }
            }
            (&Method::GET, "/bitcoin/api/check") => {
                // curl 127.0.0.1:7878/bitcoin/api/check?tb1qm4safqvzu28jvjz5juta7qutfaqst7nsfsumuz:0
                let mut query = request.uri().query().unwrap_or("").split(':');
                let addr = query.next().ok_or_else(|| gen_err())?;
                let height = query.next().ok_or_else(|| gen_err())?;
                let h: usize = height.parse::<usize>().map_err(|_| gen_err())?;

                let client = Client::new(&conf.electrum_opts.electrum).map_err(|_| gen_err())?;
                let list = check_address(&client, &addr, Option::from(h));
                match list {
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
                    Err(e) => Ok(response.body(format!("{:?}", e).as_bytes().to_vec())?),
                }
            }
            (&Method::GET, "/bitcoin/") => {
                let address = match request.uri().query() {
                    Some(address) => address,
                    None => return Ok(response.body(not_found().as_bytes().to_vec())?),
                };
                match is_my_address(&*wallet, address) {
                    Ok(mine) => {
                        if !mine {
                            return Ok(response.body(
                                format!("Address {} is not mine", address)
                                    .as_bytes()
                                    .to_vec(),
                            )?);
                        }
                    }
                    Err(e) => return Ok(response.body(format!("{:?}", e).as_bytes().to_vec())?),
                };
                match html(
                    &conf.network.to_string(),
                    &conf.electrum_opts.electrum,
                    address,
                ) {
                    Ok(txt) => Ok(response.body(txt.as_bytes().to_vec())?),
                    Err(e) => Ok(response.body(format!("{:?}", e).as_bytes().to_vec())?),
                }
            }
            (&Method::GET, "/bitcoin") => {
                let address = last_unused_address(&*wallet).map_err(|_| gen_err())?;
                let link = format!("/bitcoin/?{}", address);
                let redirect = html::redirect(link.as_str());
                match redirect {
                    Ok(txt) => Ok(response.body(txt.as_bytes().to_vec())?),
                    Err(e) => Ok(response.body(e.to_string().as_bytes().to_vec())?),
                }
            }
            (&Method::GET, "/") => {
                let link = "/bitcoin";
                let redirect = html::redirect(link);
                match redirect {
                    Ok(txt) => Ok(response.body(txt.as_bytes().to_vec())?),
                    Err(e) => Ok(response.body(e.to_string().as_bytes().to_vec())?),
                }
            }
            (_, _) => {
                response.status(StatusCode::NOT_FOUND);
                Ok(response.body(not_found().as_bytes().to_vec())?)
            }
        }
    })
}

fn last_unused_address(wallet: &Wallet<AnyBlockchain, Tree>) -> Result<Address, bdk::Error> {
    wallet.sync(log_progress(), None)?;
    wallet.get_address(LastUnused)
}

fn is_my_address(
    wallet: &Wallet<AnyBlockchain, Tree>,
    addr: &str,
) -> Result<bool, simple_server::Error> {
    let script = Address::from_str(addr)
        .map_err(|_| gen_err())?
        .script_pubkey();
    if wallet.is_mine(&script).map_err(|_| gen_err())? {
        return Ok(true);
    }
    wallet.sync(log_progress(), None).map_err(|_| gen_err())?;
    wallet.is_mine(&script).map_err(|_| gen_err())
}

fn check_address(
    client: &Client,
    addr: &str,
    from_height: Option<usize>,
) -> Result<Vec<ListUnspentRes>, simple_server::Error> {
    let monitor_script = Address::from_str(addr)
        .map_err(|_| gen_err())?
        .script_pubkey();

    let unspents = client
        .script_list_unspent(&monitor_script)
        .map_err(|_| gen_err())?;

    let array = unspents
        .into_iter()
        .filter(|x| x.height >= from_height.unwrap_or(0))
        .collect();

    Ok(array)
}

fn html(network: &str, electrum: &str, address: &str) -> Result<String, simple_server::Error> {
    let client = Client::new(electrum).map_err(|_| gen_err())?;
    let list = check_address(&client, &address, Option::from(0)).map_err(|_| gen_err())?;

    let status = match list.last() {
        None => "No tx found yet".to_string(),
        Some(unspent) => {
            let location = match unspent.height {
                0 => "in mempool".to_string(),
                _ => format!("at {}", unspent.height),
            };

            format!("Received {} sat {}", unspent.value, location)
        }
    };
    html::page(network, address, status.as_str())
}
