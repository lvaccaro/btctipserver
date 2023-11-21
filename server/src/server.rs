use http::Uri;
use simple_server::{Method, Server, StatusCode};
use std::sync::{Arc, Mutex};

use crate::{html, wallet};
use crate::html::not_found;
use std::io;
use wallet::Wallet;

/// Returns a generic simple_server::Error, used to catch errors to prevent tearing
/// down the server with a simple request, should be removed in favor of specific errors
pub fn gen_err() -> simple_server::Error {
    simple_server::Error::Io(io::Error::new(io::ErrorKind::Other, "oh no!"))
}

pub fn create_server(wallet: Wallet) -> Server {
    let wallet_mutex = Arc::new(Mutex::new(wallet));
    Server::new(move |request, mut response| {
        debug!("Request: {} {}", request.method(), request.uri());

        debug!("Headers:");
        for (key, value) in request.headers() {
            debug!("{}: {}", key, value.to_str().unwrap_or("can't map to str"));
        }
        let mut wallet_lock = wallet_mutex.lock().unwrap();
        match (request.method(), request.uri().path()) {
            (&Method::GET, "/") => {
                if request.uri().query().is_none() {
                    let html = match redirect(&mut wallet_lock) {
                        Ok(txt) => txt,
                        Err(_e) => not_found(),
                    };
                    drop(wallet_lock);
                    Ok(response.body(html.as_bytes().to_vec())?)
                } else {
                    let html = match page(&mut wallet_lock, request.uri()) {
                        Ok(txt) => txt,
                        Err(_e) => not_found(),
                    };
                    drop(wallet_lock);
                    Ok(response.body(html.as_bytes().to_vec())?)
                }
            }
            (_, _) => {
                drop(wallet_lock);
                response.status(StatusCode::NOT_FOUND);
                Ok(response.body(not_found().as_bytes().to_vec())?)
            }
        }
    })
}

pub fn redirect(
    wallet: &mut Wallet
) -> Result<String, simple_server::Error> {
    let address = wallet.last_unused_address()?;
    let link = format!("/?{}", address);
    html::redirect(link.as_str()).map_err(|_| gen_err())
}

pub fn page(
    wallet: &mut Wallet,
    uri: &Uri,
) -> Result<String, simple_server::Error> {
    let network = wallet.network()?;
    let address = uri.query().unwrap();
    let mine = wallet.is_my_address(address)?;
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
    html::page(network.as_str(), address, txt.as_str())
}
