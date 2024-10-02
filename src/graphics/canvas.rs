//! 2D drawing.

pub use femtovg::imgref::ImgRef;
use femtovg::{
    imgref::ImgVec, renderer::OpenGl, rgb::Rgba, FontId, ImageFlags, ImageId, PixelFormat,
    Transform2D,
};
use rustybuzz::ttf_parser::Face;
use std::{
    collections::HashMap,
    ffi::c_void,
    marker::PhantomData,
    ops::{Deref, DerefMut, Mul, MulAssign},
    sync::{Arc, Mutex, Weak},
};

pub(crate) enum FontInner {
    Pending(Vec<u8>),
    Loaded(FontId),
}

/// A font that can be used to draw text.
pub struct Font {
    pub(crate) inner: Mutex<FontInner>,
    metrics: FontMetrics,
}

/// Metrics for a font.
#[derive(Clone)]
pub struct FontMetrics {
    /// The recommended distance above the baseline.
    pub ascender: i16,

    /// The recommended distance below the baseline.
    pub descender: i16,

    /// Units per em.
    pub units_per_em: u16,
}

/// Metrics for some measured text.
pub struct TextMetrics {
    /// Width of the text.
    pub width: f32,

    /// Height of the text.
    pub height: f32,
}

/// Error for when font loading fails.
pub use rustybuzz::ttf_parser::FaceParsingError;

impl Font {
    /// Load a font from raw TrueType bytes.
    pub fn load(raw: &[u8]) -> Result<Self, FaceParsingError> {
        let face = Face::parse(raw, 0)?;
        Ok(Self {
            inner: Mutex::new(FontInner::Pending(raw.to_vec())),
            metrics: FontMetrics {
                ascender: face.ascender(),
                descender: face.descender(),
                units_per_em: face.units_per_em(),
            },
        })
    }

    /// Gets the metrics of the font.
    pub fn metrics(&self) -> FontMetrics {
        self.metrics.clone()
    }
}

/// A 3x2 transformation matrix representing an affine transform.
///
/// In other words, it is a 2x2 transformation matrix with a translation component, or a 3x3 homogenous transform matrix.
#[derive(Clone)]
pub struct AffineTransform([f32; 6]);

impl AffineTransform {
    /// Creates an identity transform that does nothing.
    pub const fn identity() -> Self {
        Self([
            1.0, 0.0, //
            0.0, 1.0, //
            0.0, 0.0,
        ])
    }

    /// Creates a matrix from each individual element.
    ///
    /// The matrix is in column-major order, that is:
    ///
    /// $$
    /// \begin{bmatrix}
    /// \texttt{m00} & \texttt{m10} & \texttt{tx} \\\\
    /// \texttt{m01} & \texttt{m11} & \texttt{ty} \\\\
    /// 0 & 0 & 1
    /// \end{bmatrix}
    /// $$
    pub const fn new(m00: f32, m01: f32, m10: f32, m11: f32, tx: f32, ty: f32) -> Self {
        Self([
            m00, m01, //
            m10, m11, //
            tx, ty,
        ])
    }

    /// Creates a transform that performs a translation.
    ///
    /// $$
    /// \begin{bmatrix}
    /// 1 & 0 & \texttt{tx} \\\\
    /// 0 & 1 & \texttt{ty} \\\\
    /// 0 & 0 & 1
    /// \end{bmatrix}
    /// $$
    pub const fn translation(tx: f32, ty: f32) -> Self {
        Self([
            1.0, 0.0, //
            0.0, 1.0, //
            tx, ty,
        ])
    }

    /// Creates a transform that performs scaling.
    ///
    /// $$
    /// \begin{bmatrix}
    /// \texttt{sx} & 0 & 0 \\\\
    /// 0 & \texttt{sy} & 0 \\\\
    /// 0 & 0 & 1
    /// \end{bmatrix}
    /// $$
    pub const fn scaling(sx: f32, sy: f32) -> Self {
        Self([
            sx, 0.0, //
            0.0, sy, //
            0.0, 0.0,
        ])
    }

    /// Creates a transform that performs a rotation.
    ///
    /// $$
    /// \begin{bmatrix}
    /// \text{cos}\ \theta & -\text{sin}\ \theta & 0 \\\\
    /// \text{sin}\ \theta & \text{cos}\ \theta & 0 \\\\
    /// 0 & 0 & 1
    /// \end{bmatrix}
    /// $$
    pub fn rotation(theta: f32) -> Self {
        let c = theta.cos();
        let s = theta.sin();

        Self([
            c, s, //
            -s, c, //
            0.0, 0.0,
        ])
    }

    /// Computes the determinant of the matrix.
    pub const fn determinant(&self) -> f32 {
        self.0[0] * self.0[3] - self.0[1] * self.0[2]
    }

    /// Computes the inverse of the matrix.
    ///
    /// If the matrix is degenerate (that is, the determinant is zero), returns [`None`].
    pub const fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det == 0.0 {
            return None;
        }

        let inv_det = 1.0 / det;
        Some(Self([
            inv_det * self.0[3],
            inv_det * -self.0[1],
            inv_det * -self.0[2],
            inv_det * self.0[0],
            inv_det * (self.0[1] * self.0[5] - self.0[3] * self.0[4]),
            inv_det * (self.0[4] * self.0[2] - self.0[0] * self.0[5]),
        ]))
    }

    /// Transforms a point by the matrix.
    pub const fn transform(&self, x: f32, y: f32) -> (f32, f32) {
        (
            x * self.0[0] + y * self.0[2] + self.0[4],
            x * self.0[1] + y * self.0[3] + self.0[5],
        )
    }

    /// Returns the underlying array in column-major order.
    pub const fn as_array(&self) -> &[f32; 6] {
        &self.0
    }
}

impl MulAssign<AffineTransform> for AffineTransform {
    fn mul_assign(&mut self, rhs: AffineTransform) {
        self.0 = [
            self.0[0] * rhs.0[0] + self.0[1] * rhs.0[2],
            self.0[0] * rhs.0[1] + self.0[1] * rhs.0[3],
            self.0[2] * rhs.0[0] + self.0[3] * rhs.0[2],
            self.0[2] * rhs.0[1] + self.0[3] * rhs.0[3],
            self.0[4] * rhs.0[0] + self.0[5] * rhs.0[2] + rhs.0[4],
            self.0[4] * rhs.0[1] + self.0[5] * rhs.0[3] + rhs.0[5],
        ];
    }
}

impl Mul<AffineTransform> for AffineTransform {
    type Output = AffineTransform;

    fn mul(mut self, rhs: AffineTransform) -> Self::Output {
        self *= rhs;
        self
    }
}

struct PendingTextureUpdate {
    buf: ImgVec<Rgba<u8>>,
    x: usize,
    y: usize,
}

/// A texture that can be rendered to the screen.
pub struct Texture {
    id: Mutex<Option<ImageId>>,
    ref_counter: Arc<()>,
    width: usize,
    height: usize,
    flip_y: bool,

    pending_update: Mutex<Option<PendingTextureUpdate>>,
}

impl Texture {
    fn new(width: usize, height: usize, flip_y: bool) -> Self {
        Self {
            id: Mutex::new(None),
            ref_counter: Arc::new(()),
            width,
            height,
            flip_y,
            pending_update: Mutex::new(None),
        }
    }

    /// Creates a new empty texture.
    pub fn new_empty(width: usize, height: usize) -> Self {
        Self::new(width, height, false)
    }

    /// Creates a new empty texture for use as a framebuffer.
    ///
    /// This is the same as [`Texture::new_empty`] except the Y direction is flipped.
    pub fn new_framebuffer(width: usize, height: usize) -> Self {
        Self::new(width, height, true)
    }

    /// Updates the data in the texture.
    pub fn update(&mut self, src: ImgRef<Rgba<u8>>, x: usize, y: usize) {
        // NOTE: self doesn't really need to be mutable here, but it's more morally correct.
        if x + src.width() > self.width || y + src.height() > self.height {
            // TODO: Return an error.
        }

        *self.pending_update.lock().unwrap() = Some(PendingTextureUpdate {
            buf: ImgVec::new(src.into_buf().to_vec(), src.width(), src.height()),
            x,
            y,
        });
    }

    /// Creates a texture from existing image data.
    pub fn from_data(src: ImgRef<Rgba<u8>>) -> Self {
        let mut img = Self::new_empty(src.width(), src.height());
        img.update(src, 0, 0);
        img
    }

    fn get_image_id(&self, canvas: &mut Canvas) -> femtovg::ImageId {
        let id = *self.id.lock().unwrap().get_or_insert_with(|| {
            let mut flags = ImageFlags::NEAREST;
            if self.flip_y {
                flags |= ImageFlags::FLIP_Y;
            }

            let id = canvas
                .inner
                .create_image_empty(
                    self.width as usize,
                    self.height as usize,
                    PixelFormat::Rgba8,
                    flags,
                )
                .unwrap();
            canvas.image_id_cache.insert(
                self.ref_counter.as_ref() as *const _ as *const c_void,
                CachedImageId {
                    weak: Arc::downgrade(&self.ref_counter),
                    id,
                },
            );
            id
        });

        if let Some(update) = self.pending_update.lock().unwrap().take() {
            canvas
                .inner
                .update_image(id, update.buf.as_ref(), update.x, update.y)
                .unwrap();
        }
        id
    }

    /// Gets the size of the texture.
    pub fn size(&self) -> (u32, u32) {
        (self.width as u32, self.height as u32)
    }
}

struct CachedImageId {
    weak: Weak<()>,
    id: ImageId,
}

/// Guard that on [`drop`] will undo the current transform.
pub struct CanvasTransformGuard<'a> {
    canvas: &'a mut Canvas,
}

impl<'a> CanvasTransformGuard<'a> {
    fn new(canvas: &'a mut Canvas, t: &AffineTransform) -> Self {
        canvas.inner.save();
        canvas.inner.set_transform(&Transform2D(t.0.clone()));
        Self { canvas }
    }
}

impl<'a> Drop for CanvasTransformGuard<'a> {
    fn drop(&mut self) {
        self.canvas.inner.restore();
    }
}

impl<'a> Deref for CanvasTransformGuard<'a> {
    type Target = Canvas;

    fn deref(&self) -> &Self::Target {
        self.canvas
    }
}

impl<'a> DerefMut for CanvasTransformGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.canvas
    }
}

/// Guard that on [`drop`] will switch to the previous framebuffer.
pub struct CanvasFramebufferGuard<'t, 'a> {
    canvas: &'a mut Canvas,
    prev_fb: Option<ImageId>,
    _phantom: PhantomData<&'t Texture>,
}

impl<'t, 'a> CanvasFramebufferGuard<'t, 'a> {
    fn new(canvas: &'a mut Canvas, fb: &'t Texture) -> Self {
        let prev_fb = canvas.framebuffer.take();
        let id = fb.get_image_id(canvas);
        canvas
            .inner
            .set_render_target(femtovg::RenderTarget::Image(id));
        canvas.framebuffer = Some(fb.get_image_id(canvas));
        Self {
            canvas,
            prev_fb,
            _phantom: PhantomData,
        }
    }
}

impl<'t, 'a> Drop for CanvasFramebufferGuard<'t, 'a> {
    fn drop(&mut self) {
        self.canvas
            .inner
            .set_render_target(if let Some(fb) = &self.prev_fb {
                femtovg::RenderTarget::Image(*fb)
            } else {
                femtovg::RenderTarget::Screen
            });
        self.canvas.framebuffer = self.prev_fb.take();
    }
}

impl<'t, 'a> Deref for CanvasFramebufferGuard<'t, 'a> {
    type Target = Canvas;

    fn deref(&self) -> &Self::Target {
        self.canvas
    }
}

impl<'t, 'a> DerefMut for CanvasFramebufferGuard<'t, 'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.canvas
    }
}

/// Canvas for drawing on.
pub struct Canvas {
    inner: femtovg::Canvas<OpenGl>,
    size: (u32, u32),
    framebuffer: Option<ImageId>,
    image_id_cache: HashMap<*const c_void, CachedImageId>,
}

impl Canvas {
    pub(super) fn new(canvas: femtovg::Canvas<OpenGl>) -> Self {
        Self {
            inner: canvas,
            size: (0, 0),
            framebuffer: None,
            image_id_cache: HashMap::new(),
        }
    }

    /// Gets the current size of the canvas.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub(super) fn set_size(&mut self, width: u32, height: u32, dpi: f32) {
        self.size = (width, height);
        self.inner.set_size(width, height, dpi);
    }

    pub(super) fn flush(&mut self) {
        self.inner.flush();
        self.image_id_cache.retain(|_, c| {
            if c.weak.strong_count() > 0 {
                return true;
            }

            self.inner.delete_image(c.id);
            false
        });
    }

    /// Uses a texture as a framebuffer to render onto.
    pub fn use_framebuffer<'t>(&mut self, fb: &'t Texture) -> CanvasFramebufferGuard<'t, '_> {
        CanvasFramebufferGuard::<'t, '_>::new(self, fb)
    }

    /// Uses a transform to draw with.
    pub fn transform(&mut self, t: &AffineTransform) -> CanvasTransformGuard {
        CanvasTransformGuard::new(self, t)
    }

    /// Draws an image at a given position.
    #[inline]
    pub fn draw_image(&mut self, img: &Texture, x: f32, y: f32) {
        self.draw_image_blend(img, x, y, Default::default());
    }

    /// Draws an image at a given position with a blend mode.
    #[inline]
    pub fn draw_image_blend(&mut self, img: &Texture, x: f32, y: f32, blend_mode: BlendMode) {
        let (iw, ih) = img.size();
        self.draw_image_destination_scale_blend(img, x, y, iw as f32, ih as f32, blend_mode);
    }

    /// Draws an image at a given position and scaling.
    #[inline]
    pub fn draw_image_destination_scale(
        &mut self,
        img: &Texture,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.draw_image_destination_scale_blend(img, x, y, width, height, Default::default());
    }

    /// Draws an image at a given position and scaling with a blend mode.
    #[inline]
    pub fn draw_image_destination_scale_blend(
        &mut self,
        img: &Texture,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        blend_mode: BlendMode,
    ) {
        let (iw, ih) = img.size();
        self.draw_image_source_clip_destination_scale_blend(
            img, 0.0, 0.0, iw as f32, ih as f32, x, y, width, height, blend_mode,
        );
    }

    /// Draws a subimage at a given position and scaling.
    #[inline]
    pub fn draw_image_source_clip_destination_scale(
        &mut self,
        img: &Texture,
        sx: f32,
        sy: f32,
        s_width: f32,
        s_height: f32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.draw_image_source_clip_destination_scale_blend(
            img,
            sx,
            sy,
            s_width,
            s_height,
            x,
            y,
            width,
            height,
            Default::default(),
        );
    }

    /// Draws a subimage at a given position and scaling with a blend mode.
    pub fn draw_image_source_clip_destination_scale_blend(
        &mut self,
        img: &Texture,
        sx: f32,
        sy: f32,
        s_width: f32,
        s_height: f32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        blend_mode: BlendMode,
    ) {
        self.set_blend_mode(blend_mode);
        let (iw, ih) = img.size();
        let id = img.get_image_id(self);
        self.inner.fill_path(
            &Path::new().rect(x, y, width, height).0,
            &femtovg::Paint::image(
                id,
                x - (sx * width / s_width),
                y - (sy * height / s_height),
                iw as f32 * width / s_width,
                ih as f32 * height / s_height,
                0.0,
                1.0,
            )
            .with_anti_alias(false),
        );
    }

    fn get_font_id(&mut self, font: &Font) -> FontId {
        let mut inner = font.inner.lock().unwrap();
        match &*inner {
            FontInner::Pending(vec) => {
                // It is safe to unwrap here, because we've already parsed the font once before.
                let font_id = self.inner.add_font_mem(&vec).unwrap();
                *inner = FontInner::Loaded(font_id);
                font_id
            }
            FontInner::Loaded(font_id) => *font_id,
        }
    }

    /// Strokes text at a given position.
    pub fn stroke_text(
        &mut self,
        x: f32,
        y: f32,
        text: impl AsRef<str>,
        style: &TextStyle,
        stroke: &Stroke,
        paint: &Paint,
    ) {
        self.set_blend_mode(paint.blend_mode);
        let mut impl_paint = paint.to_impl();
        style.apply_to_paint(&mut impl_paint, self);
        stroke.apply_to_paint(&mut impl_paint);
        self.inner
            .stroke_text(x, y, text, &impl_paint.with_anti_alias(false))
            .unwrap();
    }

    /// Fills text at a given position.
    pub fn fill_text(
        &mut self,
        x: f32,
        y: f32,
        text: impl AsRef<str>,
        style: &TextStyle,
        paint: &Paint,
    ) {
        self.set_blend_mode(paint.blend_mode);
        let mut impl_paint = paint.to_impl();
        style.apply_to_paint(&mut impl_paint, self);
        self.inner
            .fill_text(x, y, text, &impl_paint.with_anti_alias(false))
            .unwrap();
    }

    /// Clears a rectangle with a color at a given position.
    ///
    /// Note that this is unaffected by transforms.
    pub fn clear_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: Color) {
        self.inner.clear_rect(
            x,
            y,
            width,
            height,
            femtovg::Color::rgba(color.r, color.g, color.b, color.a),
        );
    }

    fn set_blend_mode(&mut self, BlendMode { sfactor, dfactor }: BlendMode) {
        self.inner
            .global_composite_blend_func(sfactor.to_impl(), dfactor.to_impl());
    }

    /// Fills a path with a paint.
    pub fn fill_path(&mut self, path: &Path, paint: &Paint) {
        self.set_blend_mode(paint.blend_mode);
        self.inner.fill_path(&path.0, &paint.to_impl());
    }

    /// Strokes a path with a stroke and paint.
    pub fn stroke_path(&mut self, path: &Path, stroke: &Stroke, paint: &Paint) {
        self.set_blend_mode(paint.blend_mode);
        let mut impl_paint = paint.to_impl();
        stroke.apply_to_paint(&mut impl_paint);
        self.inner.stroke_path(&path.0, &impl_paint);
    }

    /// Measure the given text using the given style.
    pub fn measure_text(&mut self, text: impl AsRef<str>, style: &TextStyle) -> TextMetrics {
        let mut impl_paint = femtovg::Paint::default();
        style.apply_to_paint(&mut impl_paint, self);
        let metrics = self
            .inner
            .measure_text(0.0, 0.0, text, &impl_paint)
            .unwrap();

        TextMetrics {
            width: metrics.width(),
            height: metrics.height(),
        }
    }
}

/// An RGBA color.
pub type Color = Rgba<u8>;

/// Treatment for beginning and end of paths.
#[derive(Default, Copy, Clone)]
pub enum LineCap {
    #[default]
    /// The stroke ends with the path, and does not project beyond it.
    Butt,

    /// The stroke projects out as a semicircle, with the center at the end of the path.
    Round,

    /// The stroke projects out as a square, with the center at the end of the path.
    Square,
}

impl LineCap {
    fn into_impl(self) -> femtovg::LineCap {
        match self {
            Self::Butt => femtovg::LineCap::Butt,
            Self::Round => femtovg::LineCap::Round,
            Self::Square => femtovg::LineCap::Square,
        }
    }
}

/// Treatment for where lines join on a stroked path.
#[derive(Default, Copy, Clone)]
pub enum LineJoin {
    #[default]
    /// The outer edges of a join meet at a sharp angle.
    Miter,

    /// The outer edges of a join meet in a circular arc.
    Round,

    /// The outer edges of a join meet with a straight line.
    Bevel,
}

impl LineJoin {
    fn into_impl(self) -> femtovg::LineJoin {
        match self {
            Self::Miter => femtovg::LineJoin::Miter,
            Self::Round => femtovg::LineJoin::Round,
            Self::Bevel => femtovg::LineJoin::Bevel,
        }
    }
}

/// Determines how blend factors are computed.
#[derive(Copy, Clone)]
pub enum BlendFactor {
    /// $(0, 0, 0, 0)$
    Zero,

    /// $(1, 1, 1, 1)$
    One,

    /// $(\frac{R_s}{k_R}, \frac{G_s}{k_G}, \frac{B_s}{k_B}, \frac{A_s}{k_A})$
    SrcColor,

    /// $(1 - \frac{R_s}{k_R}, 1 - \frac{G_s}{k_G}, 1 - \frac{B_s}{k_B}, 1 - \frac{A_s}{k_A})$
    OneMinusSrcColor,

    /// $(\frac{R_d}{k_R}, \frac{G_d}{k_G}, \frac{B_d}{k_B}, \frac{A_d}{k_A})$
    DstColor,

    /// $(1 - \frac{R_d}{k_R}, 1 - \frac{G_d}{k_G}, 1 - \frac{B_d}{k_B}, 1 - \frac{A_d}{k_A})$
    OneMinusDstColor,

    /// $(\frac{A_s}{k_a}, \frac{A_s}{k_a}, \frac{A_s}{k_a}, \frac{A_s}{k_a})$
    SrcAlpha,

    /// $(1 - \frac{A_s}{k_a}, 1 - \frac{A_s}{k_a}, 1 - \frac{A_s}{k_a}, 1 - \frac{A_s}{k_a})$
    OneMinusSrcAlpha,

    /// $(\frac{A_d}{k_a}, \frac{A_d}{k_a}, \frac{A_d}{k_a}, \frac{A_d}{k_a})$
    DstAlpha,

    /// $(1 - \frac{A_d}{k_a}, 1 - \frac{A_d}{k_a}, 1 - \frac{A_d}{k_a}, 1 - \frac{A_d}{k_a})$
    OneMinusDstAlpha,

    /// $(\frac{\text{min}(A_s, k_A - A_d)}{k_A}, \frac{\text{min}(A_s, k_A - A_d)}{k_A}, \frac{\text{min}(A_s, k_A - A_d)}{k_A}, 1)$
    SrcAlphaSaturate,
}

impl BlendFactor {
    fn to_impl(self) -> femtovg::BlendFactor {
        match self {
            Self::Zero => femtovg::BlendFactor::Zero,
            Self::One => femtovg::BlendFactor::One,
            Self::SrcColor => femtovg::BlendFactor::SrcColor,
            Self::OneMinusSrcColor => femtovg::BlendFactor::OneMinusSrcColor,
            Self::DstColor => femtovg::BlendFactor::DstColor,
            Self::OneMinusDstColor => femtovg::BlendFactor::OneMinusDstColor,
            Self::SrcAlpha => femtovg::BlendFactor::SrcAlpha,
            Self::OneMinusSrcAlpha => femtovg::BlendFactor::OneMinusSrcAlpha,
            Self::DstAlpha => femtovg::BlendFactor::DstAlpha,
            Self::OneMinusDstAlpha => femtovg::BlendFactor::OneMinusDstAlpha,
            Self::SrcAlphaSaturate => femtovg::BlendFactor::SrcAlphaSaturate,
        }
    }
}

/// Blend mode for specifying how drawing should be blended.
#[derive(Clone, Copy)]
pub struct BlendMode {
    /// Computation for the source blend factor.
    pub sfactor: BlendFactor,

    /// Computation for the destination blend factor.
    pub dfactor: BlendFactor,
}

impl BlendMode {
    /// Destination pixels covered by the source are cleared to 0.
    pub const CLEAR: Self = Self {
        sfactor: BlendFactor::Zero,
        dfactor: BlendFactor::Zero,
    };

    /// The source pixels replace the destination pixels.
    pub const SRC: Self = Self {
        sfactor: BlendFactor::One,
        dfactor: BlendFactor::Zero,
    };

    /// The source pixels are drawn over the destination pixels.
    ///
    /// This is the default blend mode and does what you would expect.
    pub const SRC_OVER: Self = Self {
        sfactor: BlendFactor::One,
        dfactor: BlendFactor::OneMinusSrcAlpha,
    };

    /// The source pixels are drawn behind the destination pixels.
    pub const DST_OVER: Self = Self {
        sfactor: BlendFactor::OneMinusDstAlpha,
        dfactor: BlendFactor::One,
    };

    /// Keeps the source pixels that cover the destination pixels, discards the remaining source and destination pixels.
    pub const SRC_IN: Self = Self {
        sfactor: BlendFactor::DstAlpha,
        dfactor: BlendFactor::Zero,
    };

    /// Keeps the destination pixels that cover source pixels, discards the remaining source and destination pixels.
    pub const DST_IN: Self = Self {
        sfactor: BlendFactor::Zero,
        dfactor: BlendFactor::SrcAlpha,
    };

    /// Keeps the source pixels that do not cover destination pixels. Discards source pixels that cover destination pixels. Discards all destination pixels.
    pub const SRC_OUT: Self = Self {
        sfactor: BlendFactor::OneMinusDstAlpha,
        dfactor: BlendFactor::Zero,
    };

    /// Keeps the destination pixels that are not covered by source pixels. Discards destination pixels that are covered by source pixels. Discards all source pixels.
    pub const DST_OUT: Self = Self {
        sfactor: BlendFactor::Zero,
        dfactor: BlendFactor::OneMinusSrcAlpha,
    };

    /// The source pixels are discarded, leaving the destination intact.
    pub const DST: Self = Self {
        sfactor: BlendFactor::Zero,
        dfactor: BlendFactor::One,
    };

    /// Discards the source pixels that do not cover destination pixels. Draws remaining source pixels over destination pixels.
    pub const SRC_ATOP: Self = Self {
        sfactor: BlendFactor::DstAlpha,
        dfactor: BlendFactor::OneMinusSrcAlpha,
    };

    /// Discards the destination pixels that are not covered by source pixels. Draws remaining destination pixels over source pixels.
    pub const DST_ATOP: Self = Self {
        sfactor: BlendFactor::OneMinusDstAlpha,
        dfactor: BlendFactor::SrcAlpha,
    };

    /// Discards the source and destination pixels where source pixels cover destination pixels. Draws remaining source pixels.
    pub const ALPHA_XOR: Self = Self {
        sfactor: BlendFactor::OneMinusDstAlpha,
        dfactor: BlendFactor::OneMinusSrcAlpha,
    };
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::SRC_OVER
    }
}

/// Fill for the paint.
#[derive(Clone)]
pub enum Fill {
    /// Solid color.
    Color(Color),

    /// Linear gradient.
    LinearGradient {
        /// Start x position.
        start_x: f32,
        /// Start y position.
        start_y: f32,
        /// End x position.
        end_x: f32,
        /// End y position.
        end_y: f32,
        /// List of gradient stops, e.g.
        /// ```
        /// vec![
        ///     (0.1, RED),
        ///     (0.5, GREEN),
        ///     (1.0, BLUE),
        /// ]
        /// ```
        stops: Vec<(f32, Color)>,
    },

    /// Radial gradient.
    RadialGradient {
        /// Center x position.
        cx: f32,
        /// Center y position.
        cy: f32,
        /// Inner radius.
        in_radius: f32,
        /// Outer radius.
        out_radius: f32,
        /// List of gradient stops, e.g.
        /// ```
        /// vec![
        ///     (0.1, RED),
        ///     (0.5, GREEN),
        ///     (1.0, BLUE),
        /// ]
        /// ```
        stops: Vec<(f32, Color)>,
    },
}

/// Paint for filling paths.
#[derive(Clone)]
pub struct Paint {
    /// The type of fill.
    pub fill: Fill,

    /// The blend mode to paint with.
    pub blend_mode: BlendMode,

    /// Whether or not the filled shape should be antialiased.
    pub anti_alias: bool,
}

impl Paint {
    /// Creates a new paint with a fill.
    pub fn new(fill: Fill) -> Self {
        Self {
            fill,
            anti_alias: true,
            blend_mode: Default::default(),
        }
    }

    /// Creates a paint with a solid color.
    pub fn color(color: Color) -> Self {
        Self::new(Fill::Color(color))
    }

    /// Creates a paint with a linear gradient.
    pub fn linear_gradient(
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        stops: Vec<(f32, Color)>,
    ) -> Self {
        Self::new(Fill::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
            stops,
        })
    }

    /// Creates a paint with a radial gradient.
    pub fn radial_gradient(
        cx: f32,
        cy: f32,
        in_radius: f32,
        out_radius: f32,
        stops: Vec<(f32, Color)>,
    ) -> Self {
        Self::new(Fill::RadialGradient {
            cx,
            cy,
            in_radius,
            out_radius,
            stops,
        })
    }

    fn to_impl(&self) -> femtovg::Paint {
        let mut paint = match &self.fill {
            Fill::Color(c) => femtovg::Paint::color(femtovg::Color::rgba(c.r, c.g, c.b, c.a)),
            Fill::LinearGradient {
                start_x,
                start_y,
                end_x,
                end_y,
                stops,
            } => femtovg::Paint::linear_gradient_stops(
                *start_x,
                *start_y,
                *end_x,
                *end_y,
                stops
                    .iter()
                    .map(|(t, c)| (*t, femtovg::Color::rgba(c.r, c.g, c.b, c.a))),
            ),
            Fill::RadialGradient {
                cx,
                cy,
                in_radius,
                out_radius,
                stops,
            } => femtovg::Paint::radial_gradient_stops(
                *cx,
                *cy,
                *in_radius,
                *out_radius,
                stops
                    .iter()
                    .map(|(t, c)| (*t, femtovg::Color::rgba(c.r, c.g, c.b, c.a))),
            ),
        };
        paint.set_anti_alias(self.anti_alias);
        paint
    }
}

/// Alignment for text.
#[derive(Default, Copy, Clone)]
pub enum Align {
    #[default]
    /// Text is aligned flush to the left edge.
    Left,

    /// Text is aligned to the center and grows evenly towards both edges.
    Center,

    /// Text is aligned flush to the right edge.
    Right,
}

impl Align {
    fn into_impl(self) -> femtovg::Align {
        match self {
            Self::Left => femtovg::Align::Left,
            Self::Center => femtovg::Align::Center,
            Self::Right => femtovg::Align::Right,
        }
    }
}

/// Baseline for aligning text to.
#[derive(Default, Copy, Clone)]
pub enum Baseline {
    /// Align to the top of the em square.
    Top,

    /// Align to the middle of the em square.
    Middle,

    #[default]
    /// Align via the normal alphabetic baseline.
    Alphabetic,

    /// Align to the bottom of the em square.
    Bottom,
}

impl Baseline {
    fn into_impl(self) -> femtovg::Baseline {
        match self {
            Self::Top => femtovg::Baseline::Top,
            Self::Middle => femtovg::Baseline::Middle,
            Self::Alphabetic => femtovg::Baseline::Alphabetic,
            Self::Bottom => femtovg::Baseline::Bottom,
        }
    }
}

/// Style for drawing text.
#[derive(Clone)]
pub struct TextStyle<'a> {
    /// Font to use.
    pub font: &'a Font,

    /// Font size.
    pub size: f32,

    /// How far the letters should be spaced from each other.
    pub letter_spacing: f32,

    /// Baseline of the text.
    pub baseline: Baseline,

    /// Alignment of the text.
    pub align: Align,
}

impl<'a> TextStyle<'a> {
    /// Creates a new text style with a given font and size.
    pub fn new(font: &'a Font, size: f32) -> Self {
        Self {
            font,
            size,
            letter_spacing: Default::default(),
            baseline: Default::default(),
            align: Default::default(),
        }
    }
}

impl<'a> TextStyle<'a> {
    fn apply_to_paint(&self, paint: &mut femtovg::Paint, canvas: &mut Canvas) {
        paint.set_font(&[canvas.get_font_id(self.font)]);
        paint.set_font_size(self.size);
        paint.set_letter_spacing(self.letter_spacing);
        paint.set_text_baseline(self.baseline.into_impl());
        paint.set_text_align(self.align.into_impl());
    }
}

/// Stroke style for stroking paths.
#[derive(Default, Clone)]
pub struct Stroke {
    /// Width of the stroke.
    pub width: f32,

    /// Limit for which a sharp corner is drawn beveled.
    pub miter_limit: f32,

    /// Treatment for the beginning of the path.
    pub cap_start: LineCap,

    /// Treatment for the end of the path.
    pub cap_end: LineCap,

    /// Treatment for how lines are joined on the path.
    pub join: LineJoin,
}

impl Stroke {
    fn apply_to_paint(&self, paint: &mut femtovg::Paint) {
        paint.set_line_width(self.width);
        paint.set_miter_limit(self.miter_limit);
        paint.set_line_cap_start(self.cap_start.into_impl());
        paint.set_line_cap_end(self.cap_end.into_impl());
        paint.set_line_join(self.join.into_impl());
    }
}

/// A path that can be filled or stroked.
#[derive(Clone)]
pub struct Path(femtovg::Path);

impl Path {
    /// Creates a new empty path.
    pub fn new() -> Self {
        Self(femtovg::Path::new())
    }

    /// Starts a new sub-path.
    pub fn move_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.0.move_to(x, y);
        self
    }

    /// Adds a line to the current sub-path.
    pub fn line_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.0.line_to(x, y);
        self
    }

    /// Adds a cubic Bézier curve to the current sub-path.
    pub fn bezier_curve_to(
        &mut self,
        c1x: f32,
        c1y: f32,
        c2x: f32,
        c2y: f32,
        x: f32,
        y: f32,
    ) -> &mut Self {
        self.0.bezier_to(c1x, c1y, c2x, c2y, x, y);
        self
    }

    /// Adds a quadratic Bézier curve to the current sub-path.
    pub fn quadratic_curve_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) -> &mut Self {
        self.0.quad_to(cx, cy, x, y);
        self
    }

    /// Adds a circular arc to the current sub-path.
    pub fn arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) -> &mut Self {
        self.0.arc_to(x1, y1, x2, y2, radius);
        self
    }

    /// Adds a rectangle to the current sub-path.
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) -> &mut Self {
        self.0.rect(x, y, w, h);
        self
    }

    /// Adds an ellipse to the current sub-path.
    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32) -> &mut Self {
        self.0.ellipse(cx, cy, rx, ry);
        self
    }

    /// Adds a circle to the current sub-path.
    pub fn circle(&mut self, cx: f32, cy: f32, r: f32) -> &mut Self {
        self.0.circle(cx, cy, r);
        self
    }

    /// Closes the current sub-path by adding a straight line from the current point to the start of the current sub-path.
    pub fn close(&mut self) -> &mut Self {
        self.0.close();
        self
    }
}
