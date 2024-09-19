use std::sync::Arc;

use crate::asset::Image;

use super::drawing::{AffineTransform, BlendMode};

pub struct NinePatch<'a> {
    img: &'a Arc<Image>,
    top_margin: u32,
    right_margin: u32,
    bottom_margin: u32,
    left_margin: u32,
}

impl<'a> NinePatch<'a> {
    pub fn new(
        img: &'a Arc<Image>,
        top_margin: u32,
        right_margin: u32,
        bottom_margin: u32,
        left_margin: u32,
    ) -> Self {
        Self {
            img,
            top_margin,
            right_margin,
            bottom_margin,
            left_margin,
        }
    }
}

impl<'a> NinePatch<'a> {
    pub fn draw(
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
        canvas.draw_image_source_clip_destination_scale(
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
        canvas.draw_image_source_clip_destination_scale(
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
        canvas.draw_image_source_clip_destination_scale(
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
        canvas.draw_image_source_clip_destination_scale(
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
        canvas.draw_image_source_clip_destination_scale(
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
        canvas.draw_image_source_clip_destination_scale(
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
        canvas.draw_image_source_clip_destination_scale(
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
        canvas.draw_image_source_clip_destination_scale(
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
        canvas.draw_image_source_clip_destination_scale(
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
