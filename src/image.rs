//! Image support.

#[derive(Clone)]
pub struct Img<Pixels> {
    pixels: Pixels,
    size: glam::UVec2,
    layers: u32,
}

impl<Pixels> Img<Pixels> {
    pub fn new(pixels: Pixels, size: glam::UVec2, layers: u32) -> Self {
        Self {
            pixels,
            size,
            layers,
        }
    }

    pub fn size(&self) -> glam::UVec2 {
        self.size
    }

    pub fn layers(&self) -> u32 {
        self.layers
    }
}

impl<Pixels> Copy for Img<Pixels> where Pixels: Copy {}

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
        Img::new(self.pixels.as_slice(), self.size, self.layers)
    }
}

impl<Pixel> Img<&[Pixel]> {
    pub fn as_buf(&self) -> &[Pixel] {
        self.pixels
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
        bytemuck::cast_slice(&img.to_rgba8()).to_vec(),
        glam::uvec2(img.width(), img.height()),
        1,
    ))
}
