use lwk_wollet::elements::AssetId;
use lwk_wollet::Error;
use serde_derive::Deserialize;
use std::collections::HashMap;

pub fn gen_err() -> Error {
    Error::Generic(format!("oh no!"))
}

#[derive(Deserialize, Clone, Debug)]
pub struct Asset {
    pub asset_id: AssetId,
    pub precision: u8,
    pub name: String,
    pub ticker: String,
}

pub struct EsploraRepository {
    pub assets: HashMap<AssetId, Asset>,
}

impl EsploraRepository {
    pub fn get(&mut self, asset_id: AssetId) -> Result<Asset, Error> {
        match self.assets.get(&asset_id).as_ref() {
            Some(&asset) => Ok(asset.clone()),
            None => self.fetch(asset_id),
        }
    }

    pub fn fetch(&mut self, asset_id: AssetId) -> Result<Asset, Error> {
        let url = format!("https://blockstream.info/liquid/api/asset/{}", asset_id);
        let res = reqwest::blocking::get(url).map_err(|_| gen_err())?;
        let asset: Asset = res.json().map_err(|_| gen_err())?;
        println!("Asset: {:#?}", asset);
        self.assets.insert(asset_id, asset.clone());
        Ok(asset)
    }
}
