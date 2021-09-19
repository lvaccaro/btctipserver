use crate::config::ConfigOpts;
use simple_server::{Method, Server, StatusCode};
use std::str::from_utf8;
use std::sync::{Arc, Mutex};

use crate::btcwallet::BTCWallet;
use crate::html;
use crate::html::not_found;
use std::io;

/// Returns a generic simple_server::Error, used to catch errors to prevent tearing
/// down the server with a simple request, should be removed in favor of specific errors
pub fn gen_err() -> simple_server::Error {
    simple_server::Error::Io(io::Error::new(io::ErrorKind::Other, "oh no!"))
}

pub fn create_server(conf: ConfigOpts, btcwallet: BTCWallet) -> Server {
    let btcwallet_mutex = Arc::new(Mutex::new(btcwallet));

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

        // unlock btcwallet mutex for this request
        let btcwallet = btcwallet_mutex.lock().map_err(|_| gen_err())?;

        match (request.method(), request.uri().path()) {
            (&Method::GET, "/bitcoin/api/last_unused_qr.bmp") => {
                let address = btcwallet.last_unused_address();
                match address {
                    Ok(addr) => {
                        info!("last unused addr {}", addr.to_string());
                        let qr = html::create_bmp_qr(&addr.to_qr_uri()).map_err(|_| gen_err())?;
                        response.header("Content-type", "image/bmp");
                        Ok(response.body(qr)?)
                    }
                    Err(e) => Ok(response.body(e.to_string().as_bytes().to_vec())?),
                }
            }
            (&Method::GET, "/bitcoin/api/last_unused") => {
                let address = btcwallet.last_unused_address();
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
                let list = btcwallet.check_address(&addr, Option::from(h));
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
                match btcwallet.is_my_address(address) {
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
                match html(&conf.network.to_string(), &btcwallet, address) {
                    Ok(txt) => Ok(response.body(txt.as_bytes().to_vec())?),
                    Err(e) => Ok(response.body(format!("{:?}", e).as_bytes().to_vec())?),
                }
            }
            (&Method::GET, "/bitcoin") => {
                let address = btcwallet.last_unused_address().map_err(|_| gen_err())?;
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

fn html(
    network: &str,
    btcwallet: &BTCWallet,
    address: &str,
) -> Result<String, simple_server::Error> {
    let list = btcwallet
        .check_address(&address, Option::from(0))
        .map_err(|_| gen_err())?;

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
