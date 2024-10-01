use crate::file::read;

use super::Loadable;

pub struct Raw(pub Vec<u8>);

impl Loadable for Raw {
    async fn load(path: &str) -> Result<Self, anyhow::Error> {
        Ok(Raw(read(path).await?))
    }
}
