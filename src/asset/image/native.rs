use std::{io::Cursor, sync::Arc};

use crate::asset::{Loadable, Metadata};
use image::{ImageFormat, ImageReader};
use tokio::task::spawn_blocking;

use super::ImageAndMetadata;

pub struct Image(image::DynamicImage);

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

impl<M> Loadable for ImageAndMetadata<M>
where
    M: Metadata + Sync + Send + 'static,
{
    async fn load(path: &str) -> Result<Self, anyhow::Error> {
        let buf = tokio::fs::read(path).await?;
        let mut ir = ImageReader::new(Cursor::new(&buf[..]));
        ir.set_format(ImageFormat::from_path(path)?);
        let image = ir.decode()?;
        Ok(Self {
            image: Arc::new(Image(image)),
            metadata: M::load(&buf)?,
        })
    }
}
