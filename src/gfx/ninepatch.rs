use super::CanvasExt;

pub struct NinePatch<Handle> {
    handle: Handle,
    top_margin: usize,
    right_margin: usize,
    bottom_margin: usize,
    left_margin: usize,
}

impl<Handle> NinePatch<Handle> {
    pub fn new(
        handle: Handle,
        top_margin: usize,
        right_margin: usize,
        bottom_margin: usize,
        left_margin: usize,
    ) -> Self {
        Self {
            handle,
            top_margin,
            right_margin,
            bottom_margin,
            left_margin,
        }
    }
}

#[cfg(feature = "femtovg")]
impl NinePatch<femtovg::ImageId> {
    pub fn draw(
        &self,
        canvas: &mut super::Canvas,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) {
        let (src_width, src_height) = canvas.image_size(self.handle).unwrap();

        canvas.save_with(|canvas| {
            canvas.translate(x as f32, y as f32);

            // Top left
            canvas.draw_image_source_clip_destination_scale(
                self.handle,
                0.0,
                0.0,
                self.left_margin as f32,
                self.top_margin as f32,
                0.0,
                0.0,
                self.left_margin as f32,
                self.top_margin as f32,
            );

            // Top right
            canvas.draw_image_source_clip_destination_scale(
                self.handle,
                (src_width - self.right_margin) as f32,
                0.0,
                self.right_margin as f32,
                self.top_margin as f32,
                (width - self.right_margin) as f32,
                0.0,
                self.right_margin as f32,
                self.top_margin as f32,
            );

            // Bottom left
            canvas.draw_image_source_clip_destination_scale(
                self.handle,
                0.0,
                (src_height - self.bottom_margin) as f32,
                self.left_margin as f32,
                self.bottom_margin as f32,
                0.0,
                (height - self.bottom_margin) as f32,
                self.left_margin as f32,
                self.bottom_margin as f32,
            );

            // Bottom right
            canvas.draw_image_source_clip_destination_scale(
                self.handle,
                (src_width - self.right_margin) as f32,
                (src_height - self.bottom_margin) as f32,
                self.right_margin as f32,
                self.bottom_margin as f32,
                (width - self.right_margin) as f32,
                (height - self.bottom_margin) as f32,
                self.left_margin as f32,
                self.right_margin as f32,
            );

            // Top edge
            canvas.draw_image_source_clip_destination_scale(
                self.handle,
                self.left_margin as f32,
                0.0,
                (src_width - self.left_margin - self.right_margin) as f32,
                self.top_margin as f32,
                self.left_margin as f32,
                0.0,
                (width - self.left_margin - self.right_margin) as f32,
                self.top_margin as f32,
            );

            // Bottom edge
            canvas.draw_image_source_clip_destination_scale(
                self.handle,
                self.left_margin as f32,
                (src_height - self.bottom_margin) as f32,
                (src_width - self.left_margin - self.right_margin) as f32,
                self.top_margin as f32,
                self.left_margin as f32,
                (height - self.bottom_margin) as f32,
                (width - self.left_margin - self.right_margin) as f32,
                self.bottom_margin as f32,
            );

            // Left edge
            canvas.draw_image_source_clip_destination_scale(
                self.handle,
                0.0,
                self.top_margin as f32,
                self.left_margin as f32,
                (src_height - self.top_margin - self.bottom_margin) as f32,
                0.0,
                self.top_margin as f32,
                self.left_margin as f32,
                (height - self.top_margin - self.bottom_margin) as f32,
            );

            // Right edge
            canvas.draw_image_source_clip_destination_scale(
                self.handle,
                (src_width - self.right_margin) as f32,
                self.top_margin as f32,
                self.left_margin as f32,
                (src_height - self.top_margin - self.bottom_margin) as f32,
                (width - self.right_margin) as f32,
                self.top_margin as f32,
                self.left_margin as f32,
                (height - self.top_margin - self.bottom_margin) as f32,
            );

            // Center
            canvas.draw_image_source_clip_destination_scale(
                self.handle,
                self.left_margin as f32,
                self.top_margin as f32,
                (src_width - self.left_margin - self.right_margin) as f32,
                (src_height - self.top_margin - self.bottom_margin) as f32,
                self.left_margin as f32,
                self.top_margin as f32,
                (width - self.left_margin - self.right_margin) as f32,
                (height - self.top_margin - self.bottom_margin) as f32,
            );
        });
    }
}
