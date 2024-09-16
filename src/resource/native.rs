use super::Resource;
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

pub fn load_bytes(path: &str) -> Resource<Vec<u8>> {
    let r = Resource::pending();
    {
        let path = path.to_string();
        let r = r.clone();

        spawn(async move {
            let res = tokio::fs::read(path).await.map_err(|e| Box::new(e).into());

            let mut txn = r.0.write();
            *txn = Some(Arc::new(res));
            txn.commit();
        });
    }
    r
}

pub fn load_image(path: &str) -> Resource<Image> {
    let r = Resource::pending();
    {
        let path = path.to_string();
        let r = r.clone();

        spawn_blocking(move || {
            let res = ImageReader::open(path)
                .map_err(|e| Box::new(e).into())
                .and_then(|v| v.decode().map_err(|e| Box::new(e).into()))
                .map(|img| Image(img));

            let mut txn = r.0.write();
            *txn = Some(Arc::new(res));
            txn.commit();
        });
    }
    r
}
