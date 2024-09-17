use super::Raw;
use crate::asset::Loadable;

impl Loadable for Raw {
    async fn load(path: &str) -> Result<Self, anyhow::Error> {
        Ok(Raw(tokio::fs::read(path).await?))
    }
}
