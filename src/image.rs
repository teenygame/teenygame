//! Image support.

use rgb::FromSlice as _;

pub use imgref::Img;

/// Converts an image to a reference to the image.
pub trait AsImgRef<Pixel> {
    /// Gets the image as a reference.
    fn as_ref(&self) -> Img<&[Pixel]>;
}

impl<Pixel> AsImgRef<Pixel> for Img<&[Pixel]> {
    fn as_ref(&self) -> Img<&[Pixel]> {
        *self
    }
}

impl<Pixel> AsImgRef<Pixel> for Img<Vec<Pixel>> {
    fn as_ref(&self) -> Img<&[Pixel]> {
        imgref::ImgExt::as_ref(self)
    }
}

/// Load an image from in-memory bytes.
///
/// This will perform conversion to RGBA8.
#[cfg(feature = "image")]
pub fn load_from_memory(
    bytes: &[u8],
) -> Result<Img<Vec<crate::graphics::Color>>, image::ImageError> {
    Ok(load_from_image(&image::load_from_memory(bytes)?)?)
}

/// Load an image from a [`image::DynamicImage`].
#[cfg(feature = "image")]
pub fn load_from_image(
    img: &image::DynamicImage,
) -> Result<Img<Vec<crate::graphics::Color>>, image::ImageError> {
    Ok(Img::new(
        img.to_rgba8().as_rgba().to_vec(),
        img.width() as usize,
        img.height() as usize,
    ))
}
