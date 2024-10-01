use std::sync::Mutex;

use femtovg::FontId;

use crate::file::read;

use super::Loadable;

pub(crate) enum Inner {
    Pending(Vec<u8>),
    Loaded(FontId),
}

pub struct Font(pub(crate) Mutex<Inner>);

impl Loadable for Font {
    async fn load(path: &str) -> Result<Self, anyhow::Error> {
        Ok(Self(Mutex::new(Inner::Pending(read(path).await?))))
    }
}
