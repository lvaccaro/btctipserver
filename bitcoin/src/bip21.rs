use bdk::bitcoin::{Address, Amount, Denomination};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::collections::BTreeMap;
use std::str::FromStr;
use url::{ParseError, Url};

pub struct Bip21 {
    pub scheme: String,
    pub address: Address,
    pub amount: Option<Amount>,
    pub label: Option<String>,
    pub message: Option<String>,
}

impl Bip21 {
    pub fn as_str(&self) -> Result<String, ParseError> {
        const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ');
        let mut query = BTreeMap::new();
        if let Some(a) = &self.amount {
            query.insert("amount", a.as_btc().to_string());
        }
        if let Some(l) = &self.label {
            let encoded = utf8_percent_encode(l.as_str(), FRAGMENT).to_string();
            query.insert("label", encoded);
        }
        if let Some(m) = &self.message {
            let encoded = utf8_percent_encode(m.as_str(), FRAGMENT).to_string();
            query.insert("message", encoded);
        }
        let params = query
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<String>>();
        let url = format!(
            "{}:{}?{}",
            self.scheme,
            self.address.to_string(),
            params.join("&")
        );
        Ok(url.trim_end_matches("?").to_string())
    }

    pub fn parse(string: &str) -> Result<Self, ParseError> {
        let url = Url::parse(string)?;
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        for (k, v) in url.query_pairs().into_owned() {
            params.insert(k, v);
        }
        let scheme = url.scheme().to_string();
        let address = Address::from_str(url.path()).map_err(|_| ParseError::IdnaError)?;
        let amount = match params.get("amount") {
            None => None,
            Some(amount) => Some(
                Amount::from_str_in(amount.as_str(), Denomination::Bitcoin)
                    .map_err(|_| ParseError::IdnaError)?,
            ),
        };
        let label = params.get("label").cloned();
        let message = params.get("message").cloned();
        Ok(Bip21 {
            scheme,
            address,
            amount,
            label,
            message,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::bip21::Bip21;
    use bdk::bitcoin::{Address, Amount, Denomination};
    use std::str::FromStr;

    #[test]
    fn serialize() {
        let mut bip21 = Bip21 {
            scheme: "bitcoin".to_string(),
            address: Address::from_str("2NDxuABdg2uqk9MouV6f53acAwaY2GwKVHK").unwrap(),
            amount: None,
            label: None,
            message: None,
        };
        assert_eq!(
            "bitcoin:2NDxuABdg2uqk9MouV6f53acAwaY2GwKVHK",
            bip21.as_str().unwrap()
        );
        bip21.label = Some("Luke-Jr".to_string());
        assert_eq!(
            "bitcoin:2NDxuABdg2uqk9MouV6f53acAwaY2GwKVHK?label=Luke-Jr",
            bip21.as_str().unwrap()
        );
        bip21.amount = Some(Amount::from_str_in("20.3", Denomination::Bitcoin).unwrap());
        assert_eq!(
            "bitcoin:2NDxuABdg2uqk9MouV6f53acAwaY2GwKVHK?amount=20.3&label=Luke-Jr",
            bip21.as_str().unwrap()
        );
        bip21.amount = Some(Amount::from_str_in("50", Denomination::Bitcoin).unwrap());
        bip21.message = Some("Donation for project xyz".to_string());
        assert_eq!(
            "bitcoin:2NDxuABdg2uqk9MouV6f53acAwaY2GwKVHK?amount=50&label=Luke-Jr&message=Donation%20for%20project%20xyz",
            bip21.as_str().unwrap()
        );
    }

    #[test]
    fn deserialize() {
        let url1 = Bip21::parse("bitcoin:2NDxuABdg2uqk9MouV6f53acAwaY2GwKVHK").unwrap();
        assert_eq!(
            "bitcoin:2NDxuABdg2uqk9MouV6f53acAwaY2GwKVHK",
            url1.as_str().unwrap()
        );
        assert_eq!("bitcoin", url1.scheme);
        assert_eq!(
            "2NDxuABdg2uqk9MouV6f53acAwaY2GwKVHK",
            url1.address.to_string()
        );
        let url2 = Bip21::parse("bitcoin:2NDxuABdg2uqk9MouV6f53acAwaY2GwKVHK?amount=50&label=Luke-Jr&message=Donation%20for%20project%20xyz").unwrap();
        assert_eq!("bitcoin:2NDxuABdg2uqk9MouV6f53acAwaY2GwKVHK?amount=50&label=Luke-Jr&message=Donation%20for%20project%20xyz", url2.as_str().unwrap());
        assert_eq!(50 as f64, url2.amount.unwrap().as_btc());
        assert_eq!("Luke-Jr", url2.label.unwrap().as_str());
        assert_eq!("Donation for project xyz", url2.message.unwrap().as_str());
    }
}