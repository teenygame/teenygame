//! Nine-patch drawing.

use super::{AffineTransform, Texture};

/// Represents a nine-patch and its margins.
///
/// A nine-patch, or nine-slice, is a way to proportionally scale an image by splitting it into nine parts ([Wikipedia article](https://en.wikipedia.org/wiki/9-slice_scaling)).
pub struct NinePatch<'a> {
    /// Texture to draw.
    pub texture: &'a Texture,

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
    pub fn draw(&self, scene: &mut super::Scene<'a>, x: u32, y: u32, width: u32, height: u32) {
        let src_size = self.texture.size();

        let scene = scene.add_child(AffineTransform::translation(x as f32, y as f32));

        // Top left
        scene.draw_sprite(
            self.texture,
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
        scene.draw_sprite(
            self.texture,
            (src_size.width - self.right_margin) as f32,
            0.0,
            self.right_margin as f32,
            self.top_margin as f32,
            (width - self.right_margin) as f32,
            0.0,
            self.right_margin as f32,
            self.top_margin as f32,
        );

        // Bottom left
        scene.draw_sprite(
            self.texture,
            0.0,
            (src_size.height - self.bottom_margin) as f32,
            self.left_margin as f32,
            self.bottom_margin as f32,
            0.0,
            (height - self.bottom_margin) as f32,
            self.left_margin as f32,
            self.bottom_margin as f32,
        );

        // Bottom right
        scene.draw_sprite(
            self.texture,
            (src_size.width - self.right_margin) as f32,
            (src_size.height - self.bottom_margin) as f32,
            self.right_margin as f32,
            self.bottom_margin as f32,
            (width - self.right_margin) as f32,
            (height - self.bottom_margin) as f32,
            self.left_margin as f32,
            self.right_margin as f32,
        );

        // Top edge
        scene.draw_sprite(
            self.texture,
            self.left_margin as f32,
            0.0,
            (src_size.width - self.left_margin - self.right_margin) as f32,
            self.top_margin as f32,
            self.left_margin as f32,
            0.0,
            (width - self.left_margin - self.right_margin) as f32,
            self.top_margin as f32,
        );

        // Bottom edge
        scene.draw_sprite(
            self.texture,
            self.left_margin as f32,
            (src_size.height - self.bottom_margin) as f32,
            (src_size.width - self.left_margin - self.right_margin) as f32,
            self.top_margin as f32,
            self.left_margin as f32,
            (height - self.bottom_margin) as f32,
            (width - self.left_margin - self.right_margin) as f32,
            self.bottom_margin as f32,
        );

        // Left edge
        scene.draw_sprite(
            self.texture,
            0.0,
            self.top_margin as f32,
            self.left_margin as f32,
            (src_size.height - self.top_margin - self.bottom_margin) as f32,
            0.0,
            self.top_margin as f32,
            self.left_margin as f32,
            (height - self.top_margin - self.bottom_margin) as f32,
        );

        // Right edge
        scene.draw_sprite(
            self.texture,
            (src_size.width - self.right_margin) as f32,
            self.top_margin as f32,
            self.left_margin as f32,
            (src_size.height - self.top_margin - self.bottom_margin) as f32,
            (width - self.right_margin) as f32,
            self.top_margin as f32,
            self.left_margin as f32,
            (height - self.top_margin - self.bottom_margin) as f32,
        );

        // Center
        scene.draw_sprite(
            self.texture,
            self.left_margin as f32,
            self.top_margin as f32,
            (src_size.width - self.left_margin - self.right_margin) as f32,
            (src_size.height - self.top_margin - self.bottom_margin) as f32,
            self.left_margin as f32,
            self.top_margin as f32,
            (width - self.left_margin - self.right_margin) as f32,
            (height - self.top_margin - self.bottom_margin) as f32,
        );
    }
}
