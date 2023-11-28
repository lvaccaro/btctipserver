use btctipserver_bitcoin::bdk::bitcoin::Address;
use maud::{html, Markup, DOCTYPE};
use qr_code::bmp_monochrome::BmpError;
use qr_code::QrCode;
use std::io::Cursor;
use std::str::FromStr;

use crate::wallet;
use wallet::{Error, gen_err};

#[derive(Default)]
pub struct Page {
    pub url: String,
    pub network: String,
    pub address: String,
    pub amount: Option<String>,
    pub label: Option<String>,
    pub message: Option<String>,
    pub status: Option<String>,
}

const CSS2: &str = include_str!("../../assets/css/style.css");
const CSS1: &str = include_str!("../../assets/css/styles.css");


fn inner_header(title: &str) -> Markup {
    let header = html! {
            header.header {
                div.header__inner {
                    div.header__logo {
                        a href="//" {
                            div.logo {
                              (title)
                            }
                        }
                    }
                }
            }
    };
    return header
}

fn inner_section(text: &str) -> Markup {
    let partial = html! {
        div class="media text-muted pt-3" {
            p class="media-body pb-3 mb-0 small lh-125 border-bottom border-gray" {
                span { (text) }
            }
        }
    };
    partial
}

/// Converts `input` in base64 and returns a data url
pub fn to_data_url<T: AsRef<[u8]>>(input: T, content_type: &str) -> String {
    let base64 = base64::encode(input.as_ref());
    format!("data:{};base64,{}", content_type, base64)
}

/// Creates QR containing `message`
pub fn create_bmp_qr(message: &str) -> Result<Vec<u8>, BmpError> {
    let qr = QrCode::new(message.as_bytes()).unwrap();

    // The `.mul(3)` with pixelated rescale shouldn't be needed, however, some printers doesn't
    // recognize it resulting in a blurry image, starting with a bigger image mostly prevents the
    // issue at the cost of a bigger image size.
    let bmp = qr.to_bmp().mul(4)?.add_white_border(16)?;

    let mut cursor = Cursor::new(vec![]);
    bmp.write(&mut cursor)?;
    Ok(cursor.into_inner())
}

/// Creates QR containing `message` and encode it in data url
fn create_bmp_base64_qr(message: &str) -> Result<String, BmpError> {
    let bitmap = create_bmp_qr(message)?;
    Ok(to_data_url(bitmap, "image/bmp"))
}

pub fn not_found() -> String {
    let html = html! {
        (DOCTYPE)
        html {
            body {
                h1 {
                    "404"
                }
                p {
                    "Not found!"
                }
            }
        }
    };
    html.into_string()
}

fn address_link(network: &str, address: &str) -> Result<String, Error> {
    Ok(format!("{}:{}", network, address))
}

fn address_qr(network: &str, address: &str) -> Result<String, Error> {
    match network {
        "bitcoin" | "testnet" => Ok(Address::from_str(address)
            .map_err(|_| gen_err())?
            .to_qr_uri()),
        _ => Ok(address_link(network, address)?),
    }
}

pub fn render(page: Page) -> Result<String, Error> {
    let meta_http_content = format!("{}; URL=/?{}", 10, page.url);
    let address_link = address_link(page.network.as_str(), page.address.as_str())?;
    let address_qr = address_qr(page.network.as_str(), page.address.as_str())?;
    let qr = create_bmp_base64_qr(&address_qr).map_err(|_| gen_err())?;

    let html = html! {
        (DOCTYPE)
        html {
            head {
                meta charset="UTF-8";
                meta name="robots" content="noindex";
                meta http-equiv="Refresh" content=(meta_http_content);
                title { (page.address) }
                style { (CSS1) }
                style { (CSS2) }
            }
            body {
                div.container.center.headings--one-size {
                    (inner_header(page.network.as_str()))
                    div.content {
                        div.index-content {

                            div.framed.framed-paragraph {
                                div class="center" {
                                    img class="qr" src=(qr) { }
                                    br { }
                                    (inner_section(page.address.as_str()))
                                }
                            }
                            @if let Some(amount) = &page.amount {
                                (inner_section(format!("Amount {} sats", amount.to_string().as_str()).as_str()))
                            }
                            @if let Some(label) = &page.label {
                                (inner_section(format!("Label {}", label.to_string().as_str()).as_str()))
                            }
                            @if let Some(message) = &page.message {
                                (inner_section(format!("Message {}", message.to_string().as_str()).as_str()))
                            }
                            @if let Some(status) = &page.status {
                                (inner_section(format!("{}", status.to_string().as_str()).as_str()))
                            }
                            a href=(address_link) { "Open in wallet app" }
                        }
                    }
                }
            }
        }
    };
    Ok(html.into_string())
}

pub fn redirect(link: &str) -> Result<String, std::io::Error> {
    let meta_http_content = format!("{}; URL={}", 0, link);
    let html = html! {
        (DOCTYPE)
        html {
            head {
                meta name="robots" content="noindex";
                meta http-equiv="Refresh" content=(meta_http_content);
            }
        }
    };
    Ok(html.into_string())
}
