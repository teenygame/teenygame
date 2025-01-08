//! Image support.

/// Load an image from in-memory bytes.
///
/// This will perform conversion to RGBA8.
#[cfg(feature = "image")]
pub fn load_from_memory(bytes: &[u8]) -> Result<canvasette::Image, image::ImageError> {
    Ok(load_from_image(&image::load_from_memory(bytes)?)?)
}

/// Load an image from a [`image::DynamicImage`].
#[cfg(feature = "image")]
pub fn load_from_image(img: &image::DynamicImage) -> Result<canvasette::Image, image::ImageError> {
    Ok(canvasette::Image::new(
        bytemuck::cast_slice(&img.to_rgba8()).to_vec(),
        wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: img.width(),
                height: img.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
    ))
}
