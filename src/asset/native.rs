use super::{Loadable, Raw};
use image::ImageReader;
use tokio::task::spawn_blocking;

pub struct Image(image::DynamicImage);

impl Loadable for Raw {
    async fn load(path: &str) -> Result<Self, anyhow::Error> {
        Ok(Raw(tokio::fs::read(path).await?))
    }
}

#[cfg(feature = "femtovg")]
impl<'a> TryFrom<&'a Image> for femtovg::ImageSource<'a> {
    type Error = femtovg::ErrorKind;

    fn try_from(value: &'a Image) -> Result<Self, Self::Error> {
        femtovg::ImageSource::try_from(&value.0)
    }
}

impl Loadable for Image {
    async fn load(path: &str) -> Result<Self, anyhow::Error> {
        let path = path.to_string();
        spawn_blocking(move || {
            ImageReader::open(&path)
                .map_err(|e| Box::new(e).into())
                .and_then(|v| v.decode().map_err(|e| Box::new(e).into()))
                .map(|v| Image(v))
        })
        .await
        .unwrap()
    }
}
