use std::io::Cursor;

use kira::sound::static_sound::StaticSoundData;

use crate::file::read;

use super::Loadable;

pub struct Audio(pub(crate) StaticSoundData);

impl Loadable for Audio {
    async fn load(path: &str) -> Result<Self, anyhow::Error> {
        Ok(Self(StaticSoundData::from_cursor(Cursor::new(
            read(path).await?,
        ))?))
    }
}
