use super::Raw;
use crate::asset::Loadable;
use gloo_net::http::Request;

impl Loadable for super::Raw {
    async fn load(path: &str) -> Result<Self, anyhow::Error> {
        Ok(Raw(Request::get(&path).send().await?.binary().await?))
    }
}
