use serde::{Deserialize, de::DeserializeOwned};
use worker::*;

#[derive(Debug, Default, Deserialize)]
pub struct NamePostcodeParams {
    pub name:     Option<String>,
    pub postcode: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RatingQuery {
    #[serde(flatten)]
    pub params: NamePostcodeParams,
}

impl RatingQuery {
    pub fn validate(self) -> Result<(String, String)> {
        let name = self.params.name
            .filter(|n| !n.trim().is_empty())
            .ok_or(Error::from("missing required param: name"))?;

        let pc = self.params.postcode
            .filter(|p| !p.trim().is_empty())
            .ok_or(Error::from("missing required param: pc"))?;

        Ok((name, pc))
    }
}

pub fn query_params<T: DeserializeOwned + Default>(req: &Request) -> Result<T> {
    let url = req.url()?;
    let query = url.query().unwrap_or("");
    serde_urlencoded::from_str(query).map_err(|e| Error::from(e.to_string()))
}