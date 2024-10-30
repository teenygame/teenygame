//! Various math types.
//!
//! These are just reexported from `glam`.

pub use glam::*;

/// Creates a translation matrix to (x, y).
pub fn translation(x: f32, y: f32) -> Affine2 {
    glam::Affine2::from_translation(Vec2::new(x, y))
}
