//! Various math types and shorthand helpers.

#[doc(no_inline)]
pub use glam::*;

/// Creates a translation matrix to (x, y).
pub fn translate(x: f32, y: f32) -> Affine2 {
    glam::Affine2::from_translation(Vec2::new(x, y))
}

/// Creates a scaling matrix by (sx, sy).
pub fn scale(sx: f32, sy: f32) -> Affine2 {
    glam::Affine2::from_scale(Vec2::new(sx, sy))
}

/// Creates a rotation matrix by theta.
pub fn rotate(theta: f32) -> Affine2 {
    glam::Affine2::from_angle(theta)
}

/// Creates a uniform scaling matrix by (s, s).
pub fn uniform_scale(s: f32) -> Affine2 {
    glam::Affine2::from_scale(Vec2::new(s, s))
}
