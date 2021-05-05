use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::collections::BTreeMap;
use url::{ParseError, Url};

pub struct Bip21 {
    pub scheme: String,
    pub address: String,
    pub amount: Option<String>,
    pub label: Option<String>,
    pub message: Option<String>,
}

impl Bip21 {
    pub(crate) fn as_str(&self) -> Result<String, ParseError> {
        const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ');
        let mut query = BTreeMap::new();
        if let Some(a) = &self.amount {
            query.insert("amount", a.to_string());
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
        let url = format!("{}:{}?{}", self.scheme, self.address, params.join("&"));
        Ok(url.trim_end_matches("?").to_string())
    }

    pub(crate) fn parse(string: &str) -> Result<Self, ParseError> {
        let url = Url::parse(string)?;
        let mut params = BTreeMap::new();
        for (k, v) in url.query_pairs().into_owned() {
            params.insert(k, v);
        }
        Ok(Bip21 {
            scheme: url.scheme().to_string(),
            address: url.path().to_string(),
            amount: params.get("amount").cloned(),
            label: params.get("label").cloned(),
            message: params.get("message").cloned(),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::bip21::Bip21;

    #[test]
    fn serialize() {
        let mut bip21 = Bip21 {
            scheme: "bitcoin".to_string(),
            address: "175tWpb8K1S7NmH4Zx6rewF9WQrcZv245W".to_string(),
            amount: None,
            label: None,
            message: None,
        };
        assert_eq!(
            "bitcoin:175tWpb8K1S7NmH4Zx6rewF9WQrcZv245W",
            bip21.as_str().unwrap()
        );
        bip21.label = Some("Luke-Jr".to_string());
        assert_eq!(
            "bitcoin:175tWpb8K1S7NmH4Zx6rewF9WQrcZv245W?label=Luke-Jr",
            bip21.as_str().unwrap()
        );
        bip21.amount = Some("20.3".to_string());
        assert_eq!(
            "bitcoin:175tWpb8K1S7NmH4Zx6rewF9WQrcZv245W?amount=20.3&label=Luke-Jr",
            bip21.as_str().unwrap()
        );
        bip21.amount = Some("50".to_string());
        bip21.message = Some("Donation for project xyz".to_string());
        assert_eq!(
            "bitcoin:175tWpb8K1S7NmH4Zx6rewF9WQrcZv245W?amount=50&label=Luke-Jr&message=Donation%20for%20project%20xyz",
            bip21.as_str().unwrap()
        );
    }

    #[test]
    fn deserialize() {
        let url1 = Bip21::parse("bitcoin:175tWpb8K1S7NmH4Zx6rewF9WQrcZv245W").unwrap();
        assert_eq!(
            "bitcoin:175tWpb8K1S7NmH4Zx6rewF9WQrcZv245W",
            url1.as_str().unwrap()
        );
        assert_eq!("bitcoin", url1.scheme);
        assert_eq!("175tWpb8K1S7NmH4Zx6rewF9WQrcZv245W", url1.address);
        let url2 = Bip21::parse("bitcoin:175tWpb8K1S7NmH4Zx6rewF9WQrcZv245W?amount=50&label=Luke-Jr&message=Donation%20for%20project%20xyz").unwrap();
        assert_eq!("bitcoin:175tWpb8K1S7NmH4Zx6rewF9WQrcZv245W?amount=50&label=Luke-Jr&message=Donation%20for%20project%20xyz", url2.as_str().unwrap());
        assert_eq!("50", url2.amount.unwrap().as_str());
        assert_eq!("Luke-Jr", url2.label.unwrap().as_str());
        assert_eq!("Donation for project xyz", url2.message.unwrap().as_str());
    }
}
