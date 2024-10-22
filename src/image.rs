use rgb::FromSlice as _;

/// Load an image from in-memory bytes.
///
/// This will perform conversion to RGBA8.
pub fn load_from_memory(bytes: &[u8]) -> Result<imgref::ImgVec<rgb::Rgba<u8>>, image::ImageError> {
    let img = image::load_from_memory(bytes)?;

    Ok(imgref::Img::new(
        img.to_rgba8().as_rgba().to_vec(),
        img.width() as usize,
        img.height() as usize,
    ))
}
