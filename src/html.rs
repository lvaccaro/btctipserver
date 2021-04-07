use maud::{html, Markup, DOCTYPE};
use qr_code::QrCode;
use std::io::Cursor;

const CSS: &str = include_str!("../assets/index.css");

fn inner_header() -> Markup {
    let header = html! {
        nav.navbar.navbar-expand-lg.fixed-top.navbar-dark.bg-dark {
            a.navbar-brand href="#" { "BTC Tip Server" }
            a.navbar-brand href="https://github.com/lvaccaro/btctipserver" {
                i.bi.bi-github role="img" aria-label="GitHub" {}
            }
        }
    };
    header
}

fn inner_network(network: &str) -> Markup {
    let network = network.to_lowercase();
    let partial = html! {
        div.d-flex.align-items-center."text-white-50".rounded.box-shadow."p-3"."my-3"
        .bg-orange[network == "bitcoin"]
        .bg-purple[network == "testnet"]
        .bg-blue[network == "regtest"]
        .bg-red[network == "signet"] {
            div class="lh-100" {
                h6 class="mb-0 text-white lh-100" { (network) }
            }
        }
    };
    partial
}

fn inner_address(address: &str) -> Markup {
    let partial = html! {
        div class="media text-muted pt-3" {
            p class="media-body pb-3 mb-0 small lh-125 border-bottom border-gray" {
                strong class="d-block text-gray-dark" {
                    "Address"
                }
                span { (address) }
            }
        }
    };
    partial
}

fn inner_status(status: &str) -> Markup {
    let partial = html! {
        div class="media text-muted pt-3" {
            p class="media-body pb-3 mb-0 small lh-125 border-bottom border-gray" {
                strong class="d-block text-gray-dark" {
                    "Status"
                }
                span { (status) }
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

/// Creates QR containing `message` and encode it in data url
fn create_bmp_base64_qr(message: &str) -> Result<String, std::io::Error> {
    let qr = QrCode::new(message.as_bytes()).unwrap();

    // The `.mul(3)` with pixelated rescale shouldn't be needed, however, some printers doesn't
    // recognize it resulting in a blurry image, starting with a bigger image mostly prevents the
    // issue at the cost of a bigger image size.
    let bmp = qr.to_bmp().mul(3).unwrap();

    let mut cursor = Cursor::new(vec![]);
    bmp.write(&mut cursor).unwrap();
    Ok(to_data_url(cursor.into_inner(), "image/bmp"))
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

pub fn page(network: &str, address: &str, status: &str) -> Result<String, std::io::Error> {
    let meta_http_content = format!("{}; URL=/bitcoin/?{}", 10, address);
    let address_link = format!("bitcoin://{}", address);
    let qr = create_bmp_base64_qr(address)?;

    let html = html! {
        (DOCTYPE)
        html {
            head {
                meta charset="UTF-8";
                meta name="robots" content="noindex";
                meta http-equiv="Refresh" content=(meta_http_content);
                title { (address) }
                link rel="stylesheet" href="https://stackpath.bootstrapcdn.com/bootstrap/4.4.1/css/bootstrap.min.css" integrity="sha384-Vkoo8x4CGsO3+Hhxv8T/Q5PaXtkKtu6ug5TOeNV6gBiFeWPGFN9MuhOf23Q9Ifjh" crossorigin="anonymous" {}
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.4.0/font/bootstrap-icons.css" {}
                style { (CSS) }
            }
            body {
                (inner_header())
                main role="main" class="container" {
                    (inner_network(network))
                    div class="my-3 p-3 bg-white rounded box-shadow" {
                        (inner_address(address))
                        (inner_status(status))
                        small class="d-block text-right mt-3" {
                            a href=(address_link) { "Open in wallet" }
                        }
                    }
                    div class="my-3 p-3 bg-white rounded box-shadow" {
                        div class="d-block text-center mt-3" {
                            div class="center" {
                                img class="qr" src=(qr) { }
                            }
                        }
                        small class="d-block text-right mt-3" {
                            a href="/bitcoin" { "Get unused address" }
                        }
                    }
                }
                script src="https://code.jquery.com/jquery-3.4.1.slim.min.js" integrity="sha384-J6qa4849blE2+poT4WnyKhv5vZF5SrPo0iEjwBvKU7imGFAV0wwj1yYfoRSJoZ+n" crossorigin="anonymous" {}
                script src="https://stackpath.bootstrapcdn.com/bootstrap/4.4.1/js/bootstrap.min.js" integrity="sha384-wfSDF2E50Y2D1uUdj0O3uMBJnjuUD4Ih7YwaYd1iqfktj0Uod8GCExl3Og8ifwB6" crossorigin="anonymous" {}
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
