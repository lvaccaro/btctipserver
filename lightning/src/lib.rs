pub mod config;

pub extern crate lnsocket;
extern crate structopt;
extern crate serde_json;

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use config::ClightningOpts;

/// Errors that can be thrown by the [`Wallet`](crate::wallet::Wallet)
#[derive(Debug)]
pub enum Error {
    ConnectionClosed,
    /// Generic error
    Generic(String)
}

pub fn gen_err() -> Error {
    Error::Generic(format!("oh no!"))
}

pub struct ClightningWallet {
    socket: lnsocket::lnsocket,
    conf: ClightningOpts,
    connected : bool,
}
unsafe impl Send for ClightningWallet {}

impl ClightningWallet {
    pub fn new(conf: &ClightningOpts) -> Result<Self, Error> {
        let mut socket: lnsocket::lnsocket = unsafe {
            let sock = lnsocket::lnsocket_create();
            *sock
        };
        unsafe { lnsocket::lnsocket_genkey(&mut socket); }
        let mut wallet = ClightningWallet { 
            socket: socket,
            conf: conf.clone(),
            connected: false
        };
        wallet.connect().unwrap();
        Ok(wallet)
    }

    fn connect(&mut self) -> Result<bool, Error> {
        let res_connect = unsafe {
            let c_node_id = CString::new(self.conf.nodeid.clone()).unwrap();
            let c_host = CString::new(self.conf.host.clone()).unwrap();
            let c_proxy = CString::new(self.conf.proxy.clone()).unwrap();
            lnsocket::lnsocket_connect_tor(&mut self.socket, c_node_id.as_ptr(), c_host.as_ptr(), c_proxy.as_ptr())
        };
        if res_connect == 0 {
            return Err(Error::ConnectionClosed)
        }
        assert_eq!(res_connect, 1);
        let _res_perform_init = unsafe { 
            lnsocket::lnsocket_perform_init(&mut self.socket);
        };
        //assert_eq!(res_perform_init, 1);
        self.connected = true;
        Ok(true)
    }

    fn call(&mut self, method: String, params: String) -> Result<String, Error> {
        return match self.call_internal(method.clone(), params.clone()) {
            Ok(txt) => Ok(txt),
            Err(_) => {
                println!("reconnect");
                let res_reconnect = self.connect()?;
                assert_eq!(res_reconnect, true);
                return self.call_internal(method, params)
            }
        }
    }

    fn call_internal(&mut self, method: String, params: String) -> Result<String, Error> {
        let params_ = {
            if params.is_empty() {
                "[]".to_string()
            } else {
                params.clone()
            }
        };
        let msg = format!(
            "{{\"method\":\"{}\",\"params\":{},\"rune\":\"{}\"}}",
            method, params_, self.conf.rune
        );

        println!("{}", msg);
        let res_write = unsafe {
            let mut cmd = vec![];
            cmd.append(&mut 0x4c4fu16.to_be_bytes().to_vec());
            cmd.append(&mut 1u64.to_be_bytes().to_vec());
            cmd.append(&mut msg.as_bytes().to_vec());
            assert_eq!(cmd.len(), msg.len() + 10);
            lnsocket::lnsocket_write(&mut self.socket, cmd.as_ptr(), cmd.len() as u16)
        };
        if res_write == 0 {
            return Err(Error::ConnectionClosed)
        }
        assert_eq!(res_write, 1);

        unsafe {
            let mut output = "".to_string();
            loop {
                let mut len = 0u16;
                let mut t: u8 = 0;
                let mut typ: u16 = 0;
                let addr = &mut t as *mut u8 as usize;
                let mut uptr = addr as *mut u8;
                let res_recv = lnsocket::lnsocket_recv(&mut self.socket, &mut typ, &mut uptr, &mut len);
                if res_recv == 0 {
                    return Err(Error::ConnectionClosed)
                }
                assert_eq!(res_recv, 1);
                println!("len {}", len);
                println!("res_recv {}", res_recv);
                println!("typ {:?}", typ);

                match typ {
                    0x594d => { // terminate
                        *uptr.add(len as usize) = 0x00;
                        let string = CStr::from_ptr(uptr.offset(8) as *mut i8).to_str().unwrap().to_string();
                        output = [output, string].join("");
                        return Ok(output)
                    }
                    0x594b => { // continue
                        *uptr.add(len as usize) = 0x00;
                        let string = CStr::from_ptr(uptr.offset(8) as *mut i8).to_str().unwrap().to_string();
                        output = [output, string].join("");
                    }
                    18 => { //pong
                    }
                    _ => { // unexpected
                        return Err(Error::ConnectionClosed)
                    }
                }
            }
        }
    }

    fn disconnect(&mut self) {
        unsafe {
            lnsocket::lnsocket_destroy(&mut self.socket);
        }
    }

    fn decode(&mut self, bolt11: &str) -> Result<serde_json::Value, Error> {
        let params = format!("[\"{}\"]", bolt11);
        let resp = self.call("decode".to_string(), params).unwrap();
        println!("{}", resp);
        let result: serde_json::Value = serde_json::from_str(resp.as_str()).unwrap();
        Ok(result)
    }

    fn getinfo(&mut self) -> Result<serde_json::Value, Error> {
        let resp = self.call("getinfo".to_string(), "".to_string()).unwrap();
        println!("{}", resp);
        let result: serde_json::Value = serde_json::from_str(resp.as_str()).unwrap();
        Ok(result)
    }

    fn new_invoice(&mut self, msat: String, label: String) -> Result<serde_json::Value, Error> {
        let params = format!("[\"{}\", \"{}\", \"{}\"]",
            msat, label, "");
        let resp = self.call("invoice".to_string(), params).unwrap();
        println!("{}", resp);
        let result: serde_json::Value = serde_json::from_str(resp.as_str()).unwrap();
        Ok(result)
    }

    fn get_invoice(&mut self, payment_hash: &str) -> Result<serde_json::Value, Error> {
        let params = format!("[null,null,\"{}\"]", payment_hash);
        let resp = self.call("listinvoices".to_string(), params).unwrap();
        println!("{}", resp);
        let result: serde_json::Value = serde_json::from_str(resp.as_str()).unwrap();
        Ok(result)
    }
}
impl Drop for ClightningWallet {
    fn drop(&mut self) {
        self.disconnect();
        println!("exit");
    }
}
impl ClightningWallet {
    pub fn last_unused_address(&mut self) -> Result<String, Error> {
        //let address = self.wallet.get_new_address().map_err(|_| gen_err())?;
        if !self.connected {
            self.connect().unwrap();
        }
		//let params = format!("[\"%dmsat\", \"%s\", \"%s\"]",
		//	params.Msatoshi, label, params.Description)

        let charset = "abcdefghijklmnopqrstuvwxyz";
        let label = random_string::generate(8, charset);
        let result = self.new_invoice("any".to_string(), label).unwrap();
        println!("{}", result);
        let bolt11 = result["result"]["bolt11"].as_str().unwrap();
        Ok(bolt11.to_string())
    }

    pub fn is_my_address(&mut self, addr: &str) -> Result<bool, Error> {
        let result = self.decode(addr).unwrap();
        println!("{}", result);
        let payee = result["result"]["payee"].as_str().unwrap();
        let valid = result["result"]["valid"].as_bool().unwrap();
        Ok(valid && payee == self.conf.nodeid)
    }

    pub fn balance_address(
        &mut self,
        addr: &str,
        _from_height: Option<usize>,
    ) -> Result<HashMap<String, u64>, Error> {
        let mut balances = HashMap::new();
        let decoded = self.decode(addr).unwrap();
        let payment_hash = decoded["result"]["payment_hash"].as_str().unwrap();
        let invoices = self.get_invoice(payment_hash).unwrap();
        println!("{}", invoices);
        let invoice = &invoices["result"]["invoices"][0];
        let msat = match invoice["status"].as_str().unwrap() {
            "unpaid" => 0,
            "expired" => 0,
            "paid" => {
                let msat = invoice["amount_received_msat"].as_str().unwrap();
                let amount = &msat[0..msat.len() - 3];
                amount.parse::<u64>().unwrap()
            },
            _ => 0
        };
        balances.insert(addr.to_string(), msat);
        Ok(balances)
    }

    pub fn network(&mut self) -> Result<String, Error> {
        if !self.connected {
            self.connect().unwrap();
        }
        let result = self.getinfo().unwrap();
        let network = result["result"]["network"].as_str().unwrap();
        Ok(format!("Lightning {}", network))
    }
}
