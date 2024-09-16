use super::{Asset, Error};
use crate::futures::spawn;
use image::ImageReader;
use std::sync::Arc;
use tokio::task::spawn_blocking;

pub struct Image(image::DynamicImage);

#[cfg(feature = "femtovg")]
impl<'a> TryFrom<&'a Image> for femtovg::ImageSource<'a> {
    type Error = femtovg::ErrorKind;

    fn try_from(value: &'a Image) -> Result<Self, Self::Error> {
        femtovg::ImageSource::try_from(&value.0)
    }
}

pub fn load_bytes(path: &str) -> Asset<Vec<u8>> {
    let r = Asset::pending();
    {
        let path = path.to_string();
        let r = r.clone();

        spawn(async move {
            let res = tokio::fs::read(path)
                .await
                .map_err(|e| Error(Arc::new(Box::new(e).into())))
                .map(|v| Arc::new(v));
            *r.0.lock().unwrap() = Some(res);
        });
    }
    r
}

pub fn load_image(path: &str) -> Asset<Image> {
    let r = Asset::pending();
    {
        let path = path.to_string();
        let r = r.clone();

        spawn_blocking(move || {
            let res = ImageReader::open(path)
                .map_err(|e| Error(Arc::new(Box::new(e).into())))
                .and_then(|v| v.decode().map_err(|e| Error(Arc::new(Box::new(e).into()))))
                .map(|img| Arc::new(Image(img)));
            *r.0.lock().unwrap() = Some(res);
        });
    }
    r
}
