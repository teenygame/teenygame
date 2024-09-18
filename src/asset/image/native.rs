use std::{io::Cursor, sync::Arc};

use crate::asset::{Loadable, Metadata};
use image::{ImageFormat, ImageReader};
use tokio::task::spawn_blocking;

use super::ImageAndMetadata;

pub struct Image(image::DynamicImage);

impl Image {
    pub fn size(&self) -> (u32, u32) {
        (self.0.width(), self.0.height())
    }
}

impl<'a> From<&'a Image> for femtovg::ImageSource<'a> {
    fn from(value: &'a Image) -> Self {
        // This is safe because we check when we load the image.
        femtovg::ImageSource::try_from(&value.0).unwrap()
    }
}

impl Loadable for Image {
    async fn load(path: &str) -> Result<Self, anyhow::Error> {
        let path = path.to_string();
        spawn_blocking(move || {
            let ir = ImageReader::open(&path)?;
            let img = ir.decode()?;

            femtovg::ImageSource::try_from(&img)?;

            Ok(Image(img))
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
