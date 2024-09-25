use std::sync::Mutex;

use femtovg::FontId;

use super::{Loadable, Raw};

pub(crate) enum Inner {
    Pending(Vec<u8>),
    Loaded(FontId),
}

pub struct Font(pub(crate) Mutex<Inner>);

impl Loadable for Font {
    async fn load(path: &str) -> Result<Self, anyhow::Error> {
        Ok(Self(Mutex::new(Inner::Pending(Raw::load(path).await?.0))))
    }
}
