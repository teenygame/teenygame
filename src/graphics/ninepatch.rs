//! Nine-patch drawing.

use super::canvas::{AffineTransform, BlendMode, Texture};

/// Represents a nine-patch and its margins.
///
/// A nine-patch, or nine-slice, is a way to proportionally scale an image by splitting it into nine parts ([Wikipedia article](https://en.wikipedia.org/wiki/9-slice_scaling)).
pub struct NinePatch<'a> {
    /// Texture to draw.
    pub img: &'a Texture,

    /// Top edge.
    pub top_margin: u32,

    /// Right edge.
    pub right_margin: u32,

    /// Bottom edge.
    pub bottom_margin: u32,

    /// Left edge.
    pub left_margin: u32,
}

impl<'a> NinePatch<'a> {
    /// Draw the nine-patch to the canvas.
    pub fn draw(&self, canvas: &mut super::Canvas, x: u32, y: u32, width: u32, height: u32) {
        self.draw_blend(canvas, x, y, width, height, Default::default());
    }

    /// Draw the nine-patch to the canvas using a blend mode.
    pub fn draw_blend(
        &self,
        canvas: &mut super::Canvas,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        blend_mode: BlendMode,
    ) {
        let (src_width, src_height) = self.img.size();

        canvas.transform(&AffineTransform::translation(x as f32, y as f32));

        // Top left
        canvas.draw_image_source_clip_destination_scale_blend(
            self.img,
            0.0,
            0.0,
            self.left_margin as f32,
            self.top_margin as f32,
            0.0,
            0.0,
            self.left_margin as f32,
            self.top_margin as f32,
            blend_mode,
        );

        // Top right
        canvas.draw_image_source_clip_destination_scale_blend(
            self.img,
            (src_width - self.right_margin) as f32,
            0.0,
            self.right_margin as f32,
            self.top_margin as f32,
            (width - self.right_margin) as f32,
            0.0,
            self.right_margin as f32,
            self.top_margin as f32,
            blend_mode,
        );

        // Bottom left
        canvas.draw_image_source_clip_destination_scale_blend(
            self.img,
            0.0,
            (src_height - self.bottom_margin) as f32,
            self.left_margin as f32,
            self.bottom_margin as f32,
            0.0,
            (height - self.bottom_margin) as f32,
            self.left_margin as f32,
            self.bottom_margin as f32,
            blend_mode,
        );

        // Bottom right
        canvas.draw_image_source_clip_destination_scale_blend(
            self.img,
            (src_width - self.right_margin) as f32,
            (src_height - self.bottom_margin) as f32,
            self.right_margin as f32,
            self.bottom_margin as f32,
            (width - self.right_margin) as f32,
            (height - self.bottom_margin) as f32,
            self.left_margin as f32,
            self.right_margin as f32,
            blend_mode,
        );

        // Top edge
        canvas.draw_image_source_clip_destination_scale_blend(
            self.img,
            self.left_margin as f32,
            0.0,
            (src_width - self.left_margin - self.right_margin) as f32,
            self.top_margin as f32,
            self.left_margin as f32,
            0.0,
            (width - self.left_margin - self.right_margin) as f32,
            self.top_margin as f32,
            blend_mode,
        );

        // Bottom edge
        canvas.draw_image_source_clip_destination_scale_blend(
            self.img,
            self.left_margin as f32,
            (src_height - self.bottom_margin) as f32,
            (src_width - self.left_margin - self.right_margin) as f32,
            self.top_margin as f32,
            self.left_margin as f32,
            (height - self.bottom_margin) as f32,
            (width - self.left_margin - self.right_margin) as f32,
            self.bottom_margin as f32,
            blend_mode,
        );

        // Left edge
        canvas.draw_image_source_clip_destination_scale_blend(
            self.img,
            0.0,
            self.top_margin as f32,
            self.left_margin as f32,
            (src_height - self.top_margin - self.bottom_margin) as f32,
            0.0,
            self.top_margin as f32,
            self.left_margin as f32,
            (height - self.top_margin - self.bottom_margin) as f32,
            blend_mode,
        );

        // Right edge
        canvas.draw_image_source_clip_destination_scale_blend(
            self.img,
            (src_width - self.right_margin) as f32,
            self.top_margin as f32,
            self.left_margin as f32,
            (src_height - self.top_margin - self.bottom_margin) as f32,
            (width - self.right_margin) as f32,
            self.top_margin as f32,
            self.left_margin as f32,
            (height - self.top_margin - self.bottom_margin) as f32,
            blend_mode,
        );

        // Center
        canvas.draw_image_source_clip_destination_scale_blend(
            self.img,
            self.left_margin as f32,
            self.top_margin as f32,
            (src_width - self.left_margin - self.right_margin) as f32,
            (src_height - self.top_margin - self.bottom_margin) as f32,
            self.left_margin as f32,
            self.top_margin as f32,
            (width - self.left_margin - self.right_margin) as f32,
            (height - self.top_margin - self.bottom_margin) as f32,
            blend_mode,
        );
    }
}
