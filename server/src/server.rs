use tiny_http::{Header, Server, Response, Method};
use std::sync::{Arc, Mutex};
use uriparse;
use std::convert::TryFrom;

use crate::{html, wallet};
use crate::html::{not_found, Page};
use wallet::{Wallet, Error, gen_err};

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
    let mut page = Page {
        network: wallet.network()?,
        url: format!("{}", uri),
        address: format!("{}", uri),
        ..Default::default() 
    };

    if uri.starts_with(wallet.schema()) {
        println!("{}",wallet.schema());
        if let Ok(bip21) = btctipserver_bitcoin::bip21::Bip21::parse(uri) {
            page.address = bip21.address.to_string();
            if let Some(amount) = bip21.amount {
                page.amount = Some(amount.as_sat().to_string());
            }
            page.label = bip21.label;
            page.label = bip21.message;
        }
    }
    let mine = wallet.is_my_address(page.address.as_str())?;
    if !mine {
        return Ok(format!("Address {} is not mine", page.address));
    }
    let results = wallet
        .balance_address(&page.address, Option::from(0))
        .map_err(|_| gen_err())?
        .into_iter()
        .filter(|(_, v)| *v > 0)
        .map(|(k, v)| (k.clone(), v.clone()))
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<String>>()
        .join(", ");

    page.status = match results.is_empty() {
        true => Some("No tx found yet".to_string()),
        _ => Some(results),
    };
    html::render(page)
}
