use http::Uri;
use simple_server::{Method, Server, StatusCode};
use std::sync::{Arc, Mutex};

use crate::html;
use crate::html::not_found;
use crate::wallet::Wallet;
use std::io;
use std::sync::MutexGuard;

/// Returns a generic simple_server::Error, used to catch errors to prevent tearing
/// down the server with a simple request, should be removed in favor of specific errors
pub fn gen_err() -> simple_server::Error {
    simple_server::Error::Io(io::Error::new(io::ErrorKind::Other, "oh no!"))
}

pub fn create_server(wallet: impl Wallet + 'static) -> Server {
    let wallet_mutex = Arc::new(Mutex::new(wallet));
    Server::new(move |request, mut response| {
        debug!("Request: {} {}", request.method(), request.uri());

        debug!("Headers:");
        for (key, value) in request.headers() {
            debug!("{}: {}", key, value.to_str().unwrap_or("can't map to str"));
        }
        let wallet_lock = wallet_mutex.lock().unwrap();
        match (request.method(), request.uri().path()) {
            (&Method::GET, "/") => {
                if request.uri().query().is_none() {
                    match redirect(wallet_lock) {
                        Ok(txt) => Ok(response.body(txt.as_bytes().to_vec())?),
                        Err(_e) => Ok(response.body(not_found().as_bytes().to_vec())?),
                    }
                } else {
                    let network = wallet_lock.network().unwrap();
                    match page(wallet_lock, network.as_str(), request.uri()) {
                        Ok(txt) => Ok(response.body(txt.as_bytes().to_vec())?),
                        Err(_e) => Ok(response.body(not_found().as_bytes().to_vec())?),
                    }
                }
            }
            (_, _) => {
                response.status(StatusCode::NOT_FOUND);
                Ok(response.body(not_found().as_bytes().to_vec())?)
            }
        }
    })
}

pub fn redirect(wallet: MutexGuard<impl Wallet>) -> Result<String, simple_server::Error> {
    let address = wallet.last_unused_address().map_err(|_| gen_err())?;
    let link = format!("/?{}", address);
    html::redirect(link.as_str()).map_err(|_| gen_err())
}

pub fn page(
    wallet: MutexGuard<impl Wallet>,
    network: &str,
    uri: &Uri,
) -> Result<String, simple_server::Error> {
    let address = uri.query().unwrap();
    let mine = wallet.is_my_address(address).map_err(|_| gen_err())?;
    if !mine {
        return Ok(format!("Address {} is not mine", address));
    }
    let results = wallet
        .balance_address(&address, Option::from(0))
        .map_err(|_| gen_err())?
        .into_iter()
        .filter(|(_, v)| *v > 0)
        .map(|(k, v)| (k.clone(), v.clone()))
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<String>>()
        .join(", ");

    let txt = match results.is_empty() {
        true => "No tx found yet".to_string(),
        _ => results,
    };
    html::page(network, address, txt.as_str())
}
