use tiny_http::{Header, Server, Response, Method};
use std::sync::{Arc, Mutex};
use uriparse;
use std::convert::TryFrom;

use crate::{html, wallet};
use crate::html::not_found;
use wallet::{Wallet, Error, gen_err};

use btctipserver_bitcoin::BTCWallet;

pub fn run_server(url: &str, wallet: Wallet) {
    let wallet_mutex = Arc::new(Mutex::new(wallet));
    let server = Server::http(url).unwrap();
    for request in server.incoming_requests() {
        println!("received request! method: {:?}, url: {:?}, headers: {:?}",
            request.method(),
            request.url(),
            request.headers()
        );
        let mut wallet_lock = wallet_mutex.lock().unwrap();
        let parsed = uriparse::URIReference::try_from(request.url()).unwrap();
        let content_type_header = "Content-Type: text/html; charset=utf-8".parse::<Header>().unwrap();
        println!("query");
        println!("{}",parsed.query().unwrap().to_string().as_str());

        let response = match (request.method(), parsed.path().to_string().as_str()) {
            (&Method::Get, "/") => {
                if parsed.query().is_none() {
                    let html = match redirect(&mut wallet_lock) {
                        Ok(txt) => txt,
                        Err(_e) => not_found(),
                    };
                    drop(wallet_lock);
                    Response::from_string(html).with_header(content_type_header)
                } else {
                    let html = match page(&mut wallet_lock, parsed.query().unwrap()) {
                        Ok(txt) => txt,
                        Err(_e) => not_found(),
                    };
                    drop(wallet_lock);
                    Response::from_string(html).with_header(content_type_header)
                }
            }
            (_, _) => {
                drop(wallet_lock);
                Response::from_string(not_found()).with_status_code(404).with_header(content_type_header)
            }
        };
        request.respond(response).unwrap();
    }
}

pub fn redirect(
    wallet: &mut Wallet
) -> Result<String, Error> {
    let address = wallet.last_unused_address()?;
    let link = format!("/?{}", address);
    html::redirect(link.as_str()).map_err(|_| gen_err())
}

pub fn page(
    wallet: &mut Wallet,
    uri: &str,
) -> Result<String, Error> {
    let network = wallet.network()?;
    let mut address = uri;

    BTCWallet::Bip21::parse(uri).unwrap();


    Bip21::parse("bitcoin:2NDxuABdg2uqk9MouV6f53acAwaY2GwKVHK")
    let address = match parsed.query().unwrap() {
        
    };



    if parsed.query().unwrap().starts_with(wallet.schema()) {
        address =  Bip21::parse("bitcoin:2NDxuABdg2uqk9MouV6f53acAwaY2GwKVHK").unwrap();
    } 

    let address = uri;
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
