use std::io::Cursor;

use kira::sound::static_sound::StaticSoundData;

use super::{Loadable, Raw};

pub struct Audio(pub(crate) StaticSoundData);

impl Loadable for Audio {
    async fn load(path: &str) -> Result<Self, anyhow::Error> {
        Ok(Self(StaticSoundData::from_cursor(Cursor::new(
            Raw::load(path).await?.0,
        ))?))
    }
}
