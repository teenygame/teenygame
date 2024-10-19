//! Nine-patch drawing.

use super::{AffineTransform, TextureSlice};

/// Represents a nine-patch and its margins.
///
/// A nine-patch, or nine-slice, is a way to proportionally scale an image by splitting it into nine parts ([Wikipedia article](https://en.wikipedia.org/wiki/9-slice_scaling)).
pub struct NinePatch<'a> {
    /// Texture to draw.
    pub texture_slice: TextureSlice<'a>,

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
    pub fn draw(&self, scene: &mut super::Scene<'a>, x: i32, y: i32, width: u32, height: u32) {
        let [swidth, sheight] = self.texture_slice.size();

        let scene = scene.add_child(AffineTransform::translation(x as f32, y as f32));

        // Top left
        scene.draw_sprite(
            self.texture_slice
                .slice(0, 0, self.left_margin, self.top_margin)
                .unwrap(),
            0,
            0,
            self.left_margin,
            self.top_margin,
        );

        // Top right
        scene.draw_sprite(
            self.texture_slice
                .slice(
                    (swidth - self.right_margin) as i32,
                    0,
                    self.right_margin,
                    self.top_margin,
                )
                .unwrap(),
            (width - self.right_margin) as i32,
            0,
            self.right_margin,
            self.top_margin,
        );

        // Bottom left
        scene.draw_sprite(
            self.texture_slice
                .slice(
                    0,
                    (sheight - self.bottom_margin) as i32,
                    self.left_margin,
                    self.bottom_margin,
                )
                .unwrap(),
            0,
            (height - self.bottom_margin) as i32,
            self.left_margin,
            self.bottom_margin,
        );

        // Bottom right
        scene.draw_sprite(
            self.texture_slice
                .slice(
                    (swidth - self.right_margin) as i32,
                    (sheight - self.bottom_margin) as i32,
                    self.right_margin,
                    self.bottom_margin,
                )
                .unwrap(),
            (width - self.right_margin) as i32,
            (height - self.bottom_margin) as i32,
            self.left_margin,
            self.right_margin,
        );

        // Top edge
        scene.draw_sprite(
            self.texture_slice
                .slice(
                    self.left_margin as i32,
                    0,
                    swidth - self.left_margin - self.right_margin,
                    self.top_margin,
                )
                .unwrap(),
            self.left_margin as i32,
            0,
            width - self.left_margin - self.right_margin,
            self.top_margin,
        );

        // Bottom edge
        scene.draw_sprite(
            self.texture_slice
                .slice(
                    self.left_margin as i32,
                    (sheight - self.bottom_margin) as i32,
                    swidth - self.left_margin - self.right_margin,
                    self.top_margin,
                )
                .unwrap(),
            self.left_margin as i32,
            (height - self.bottom_margin) as i32,
            width - self.left_margin - self.right_margin,
            self.bottom_margin,
        );

        // Left edge
        scene.draw_sprite(
            self.texture_slice
                .slice(
                    0,
                    self.top_margin as i32,
                    self.left_margin,
                    sheight - self.top_margin - self.bottom_margin,
                )
                .unwrap(),
            0,
            self.top_margin as i32,
            self.left_margin,
            height - self.top_margin - self.bottom_margin,
        );

        // Right edge
        scene.draw_sprite(
            self.texture_slice
                .slice(
                    (swidth - self.right_margin) as i32,
                    self.top_margin as i32,
                    self.left_margin,
                    sheight - self.top_margin - self.bottom_margin,
                )
                .unwrap(),
            (width - self.right_margin) as i32,
            self.top_margin as i32,
            self.left_margin,
            height - self.top_margin - self.bottom_margin,
        );

        // Center
        scene.draw_sprite(
            self.texture_slice
                .slice(
                    self.left_margin as i32,
                    self.top_margin as i32,
                    swidth - self.left_margin - self.right_margin,
                    sheight - self.top_margin - self.bottom_margin,
                )
                .unwrap(),
            self.left_margin as i32,
            self.top_margin as i32,
            width - self.left_margin - self.right_margin,
            height - self.top_margin - self.bottom_margin,
        );
    }
}
