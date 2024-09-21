use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::c_void,
    ops::{Deref, DerefMut, Mul, MulAssign},
    sync::{Arc, Mutex, Weak},
};

use bytemuck::checked::cast_slice;
use femtovg::{
    imgref::ImgVec, renderer::OpenGl, rgb::Rgba, ImageFlags, ImageId, PixelFormat, Transform2D,
};

pub struct AffineTransform([f32; 6]);

impl AffineTransform {
    pub fn identity() -> Self {
        Self([
            1.0, 0.0, //
            0.0, 1.0, //
            0.0, 0.0,
        ])
    }

    pub fn new(m00: f32, m01: f32, m10: f32, m11: f32, tx: f32, ty: f32) -> Self {
        Self([
            m00, m01, //
            m10, m11, //
            tx, ty,
        ])
    }

    pub fn translation(tx: f32, ty: f32) -> Self {
        Self([
            1.0, 0.0, //
            0.0, 1.0, //
            tx, ty,
        ])
    }

    pub fn scaling(sx: f32, sy: f32) -> Self {
        Self([
            sx, 0.0, //
            0.0, sy, //
            0.0, 0.0,
        ])
    }

    pub fn rotation(theta: f32) -> Self {
        let c = theta.cos();
        let s = theta.sin();

        Self([
            c, s, //
            -s, c, //
            0.0, 0.0,
        ])
    }

    pub fn as_array(&self) -> &[f32; 6] {
        &self.0
    }
}

impl MulAssign<AffineTransform> for AffineTransform {
    fn mul_assign(&mut self, rhs: AffineTransform) {
        let t0 = self.0[0] * rhs.0[0] + self.0[1] * rhs.0[2];
        let t2 = self.0[2] * rhs.0[0] + self.0[3] * rhs.0[2];
        let t4 = self.0[4] * rhs.0[0] + self.0[5] * rhs.0[2] + rhs.0[4];
        self.0[1] = self.0[0] * rhs.0[1] + self.0[1] * rhs.0[3];
        self.0[3] = self.0[2] * rhs.0[1] + self.0[3] * rhs.0[3];
        self.0[5] = self.0[4] * rhs.0[1] + self.0[5] * rhs.0[3] + rhs.0[5];
        self.0[0] = t0;
        self.0[2] = t2;
        self.0[4] = t4;
    }
}

impl Mul<AffineTransform> for AffineTransform {
    type Output = AffineTransform;

    fn mul(mut self, rhs: AffineTransform) -> Self::Output {
        self *= rhs;
        self
    }
}

pub trait Image {
    fn get_image_id(&self, canvas: &mut Canvas) -> femtovg::ImageId;
    fn size(&self) -> (u32, u32);
}

impl Image for Arc<crate::asset::Image> {
    fn get_image_id(&self, canvas: &mut Canvas) -> femtovg::ImageId {
        canvas.get_or_create_image(self.clone(), ImageFlags::NEAREST)
    }

    fn size(&self) -> (u32, u32) {
        self.as_ref().size()
    }
}

pub struct Texture {
    id: RefCell<Option<ImageId>>,
    width: usize,
    height: usize,
    flip_y: bool,

    pending_update: Mutex<Option<(ImgVec<Rgba<u8>>, (usize, usize))>>,
}

impl Image for Arc<Texture> {
    fn get_image_id(&self, canvas: &mut Canvas) -> femtovg::ImageId {
        let id = *self.id.borrow_mut().get_or_insert_with(|| {
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
                (self.as_ref() as *const _ as *const c_void, flags),
                CachedImageId {
                    weak: Box::new(Arc::downgrade(self)),
                    id,
                },
            );
            id
        });

        if let Some((src, (x, y))) = self.pending_update.lock().unwrap().take() {
            canvas.inner.update_image(id, src.as_ref(), x, y).unwrap();
        }
        id
    }

    fn size(&self) -> (u32, u32) {
        (self.width as u32, self.height as u32)
    }
}

impl Texture {
    pub fn new(width: usize, height: usize, flip_y: bool) -> Arc<Self> {
        Arc::new(Self {
            id: RefCell::new(None),
            width,
            height,
            flip_y,
            pending_update: Mutex::new(None),
        })
    }

    pub fn new_framebuffer(width: usize, height: usize) -> Arc<Self> {
        Self::new(width, height, true)
    }

    pub fn update_rgba(&self, src: &[u8], x: usize, y: usize, width: usize, height: usize) {
        *self.pending_update.lock().unwrap() = Some((
            ImgVec::new(cast_slice::<_, Rgba<u8>>(src).to_vec(), width, height),
            (x, y),
        ));
    }
}

struct CachedImageId {
    weak: Box<dyn AnyWeak>,
    id: ImageId,
}

trait AnyWeak {
    fn strong_count(&self) -> usize;
}

impl<T> AnyWeak for Weak<T> {
    fn strong_count(&self) -> usize {
        self.strong_count()
    }
}

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

pub struct CanvasFramebufferGuard<'a> {
    canvas: &'a mut Canvas,
    prev_fb: Option<Arc<Texture>>,
}

impl<'a> CanvasFramebufferGuard<'a> {
    fn new(canvas: &'a mut Canvas, fb: Arc<Texture>) -> Self {
        let prev_fb = canvas.framebuffer.take();
        let id = fb.get_image_id(canvas);
        canvas
            .inner
            .set_render_target(femtovg::RenderTarget::Image(id));
        canvas.framebuffer = Some(fb);
        Self { canvas, prev_fb }
    }
}

impl<'a> Drop for CanvasFramebufferGuard<'a> {
    fn drop(&mut self) {
        if let Some(fb) = &self.prev_fb {
            self.canvas
                .inner
                .set_render_target(femtovg::RenderTarget::Image(fb.id.borrow().unwrap()));
        } else {
            self.canvas
                .inner
                .set_render_target(femtovg::RenderTarget::Screen);
        }
        self.canvas.framebuffer = self.prev_fb.take();
    }
}

impl<'a> Deref for CanvasFramebufferGuard<'a> {
    type Target = Canvas;

    fn deref(&self) -> &Self::Target {
        self.canvas
    }
}

impl<'a> DerefMut for CanvasFramebufferGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.canvas
    }
}
pub struct Canvas {
    inner: femtovg::Canvas<OpenGl>,
    size: (u32, u32),
    framebuffer: Option<Arc<Texture>>,
    image_id_cache: HashMap<(*const c_void, ImageFlags), CachedImageId>,
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

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    fn get_or_create_image(
        &mut self,
        img: Arc<crate::asset::Image>,
        flags: ImageFlags,
    ) -> femtovg::ImageId {
        match self
            .image_id_cache
            .entry((img.as_ref() as *const _ as *const c_void, flags))
        {
            std::collections::hash_map::Entry::Occupied(e) => e.get().id,
            std::collections::hash_map::Entry::Vacant(e) => {
                let id = self
                    .inner
                    .create_image(femtovg::ImageSource::from(&*img), flags)
                    .unwrap();
                e.insert(CachedImageId {
                    weak: Box::new(Arc::downgrade(&img)),
                    id,
                });
                id
            }
        }
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

    pub fn use_framebuffer(&mut self, fb: Arc<Texture>) -> CanvasFramebufferGuard {
        CanvasFramebufferGuard::new(self, fb)
    }

    pub fn transform(&mut self, t: &AffineTransform) -> CanvasTransformGuard {
        CanvasTransformGuard::new(self, t)
    }

    #[inline]
    pub fn draw_image<D>(&mut self, d: &D, x: f32, y: f32)
    where
        D: Image,
    {
        self.draw_image_blend(d, x, y, Default::default());
    }

    #[inline]
    pub fn draw_image_blend<D>(&mut self, d: &D, x: f32, y: f32, blend_mode: BlendMode)
    where
        D: Image,
    {
        let (iw, ih) = d.size();
        self.draw_image_destination_scale_blend(d, x, y, iw as f32, ih as f32, blend_mode);
    }

    #[inline]
    pub fn draw_image_destination_scale<D>(
        &mut self,
        d: &D,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) where
        D: Image,
    {
        self.draw_image_destination_scale_blend(d, x, y, width, height, Default::default());
    }

    #[inline]
    pub fn draw_image_destination_scale_blend<D>(
        &mut self,
        d: &D,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        blend_mode: BlendMode,
    ) where
        D: Image,
    {
        let (iw, ih) = d.size();
        self.draw_image_source_clip_destination_scale_blend(
            d, 0.0, 0.0, iw as f32, ih as f32, x, y, width, height, blend_mode,
        );
    }

    #[inline]
    pub fn draw_image_source_clip_destination_scale<D>(
        &mut self,
        d: &D,
        sx: f32,
        sy: f32,
        s_width: f32,
        s_height: f32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) where
        D: Image,
    {
        self.draw_image_source_clip_destination_scale_blend(
            d,
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

    pub fn draw_image_source_clip_destination_scale_blend<D>(
        &mut self,
        d: &D,
        sx: f32,
        sy: f32,
        s_width: f32,
        s_height: f32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        blend_mode: BlendMode,
    ) where
        D: Image,
    {
        self.set_blend_mode(blend_mode);
        let (iw, ih) = d.size();
        let id = d.get_image_id(self);
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

    pub fn clear_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: Color) {
        self.inner.clear_rect(x, y, width, height, color.into());
    }

    fn set_blend_mode(&mut self, BlendMode { sfactor, dfactor }: BlendMode) {
        self.inner
            .global_composite_blend_func(sfactor.into(), dfactor.into());
    }

    pub fn fill_path(&mut self, path: &Path, paint: &Paint) {
        self.set_blend_mode(paint.blend_mode);
        self.inner.fill_path(&path.0, &paint.to_impl_paint());
    }

    pub fn stroke_path(&mut self, path: &Path, stroke: &Stroke, paint: &Paint) {
        self.set_blend_mode(paint.blend_mode);
        let mut impl_paint = paint.to_impl_paint();
        stroke.apply_to_paint(&mut impl_paint);
        self.inner.stroke_path(&path.0, &impl_paint);
    }
}

#[derive(Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Color> for femtovg::Color {
    fn from(value: Color) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

impl From<LineCap> for femtovg::LineCap {
    fn from(value: LineCap) -> Self {
        match value {
            LineCap::Butt => femtovg::LineCap::Butt,
            LineCap::Round => femtovg::LineCap::Round,
            LineCap::Square => femtovg::LineCap::Square,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

impl From<LineJoin> for femtovg::LineJoin {
    fn from(value: LineJoin) -> Self {
        match value {
            LineJoin::Miter => femtovg::LineJoin::Miter,
            LineJoin::Round => femtovg::LineJoin::Round,
            LineJoin::Bevel => femtovg::LineJoin::Bevel,
        }
    }
}

#[derive(Copy, Clone)]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    SrcAlphaSaturate,
}

impl From<BlendFactor> for femtovg::BlendFactor {
    fn from(value: BlendFactor) -> Self {
        match value {
            BlendFactor::Zero => femtovg::BlendFactor::Zero,
            BlendFactor::One => femtovg::BlendFactor::One,
            BlendFactor::SrcColor => femtovg::BlendFactor::SrcColor,
            BlendFactor::OneMinusSrcColor => femtovg::BlendFactor::OneMinusSrcColor,
            BlendFactor::DstColor => femtovg::BlendFactor::DstColor,
            BlendFactor::OneMinusDstColor => femtovg::BlendFactor::OneMinusDstColor,
            BlendFactor::SrcAlpha => femtovg::BlendFactor::SrcAlpha,
            BlendFactor::OneMinusSrcAlpha => femtovg::BlendFactor::OneMinusSrcAlpha,
            BlendFactor::DstAlpha => femtovg::BlendFactor::DstAlpha,
            BlendFactor::OneMinusDstAlpha => femtovg::BlendFactor::OneMinusDstAlpha,
            BlendFactor::SrcAlphaSaturate => femtovg::BlendFactor::SrcAlphaSaturate,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BlendMode {
    pub sfactor: BlendFactor,
    pub dfactor: BlendFactor,
}

impl BlendMode {
    pub const CLEAR: Self = Self {
        sfactor: BlendFactor::Zero,
        dfactor: BlendFactor::Zero,
    };

    pub const SRC: Self = Self {
        sfactor: BlendFactor::One,
        dfactor: BlendFactor::Zero,
    };

    pub const SRC_OVER: Self = Self {
        sfactor: BlendFactor::One,
        dfactor: BlendFactor::OneMinusSrcAlpha,
    };

    pub const DST_OVER: Self = Self {
        sfactor: BlendFactor::OneMinusDstAlpha,
        dfactor: BlendFactor::One,
    };

    pub const SRC_IN: Self = Self {
        sfactor: BlendFactor::DstAlpha,
        dfactor: BlendFactor::Zero,
    };

    pub const DST_IN: Self = Self {
        sfactor: BlendFactor::Zero,
        dfactor: BlendFactor::SrcAlpha,
    };

    pub const SRC_OUT: Self = Self {
        sfactor: BlendFactor::OneMinusDstAlpha,
        dfactor: BlendFactor::Zero,
    };

    pub const DST_OUT: Self = Self {
        sfactor: BlendFactor::Zero,
        dfactor: BlendFactor::OneMinusSrcAlpha,
    };

    pub const DST: Self = Self {
        sfactor: BlendFactor::Zero,
        dfactor: BlendFactor::One,
    };

    pub const SRC_ATOP: Self = Self {
        sfactor: BlendFactor::DstAlpha,
        dfactor: BlendFactor::OneMinusSrcAlpha,
    };

    pub const DST_ATOP: Self = Self {
        sfactor: BlendFactor::OneMinusDstAlpha,
        dfactor: BlendFactor::SrcAlpha,
    };

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

enum PaintKind {
    Color(Color),
    LinearGradient {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        stops: Vec<(f32, Color)>,
    },
    RadialGradient {
        cx: f32,
        cy: f32,
        in_radius: f32,
        out_radius: f32,
        stops: Vec<(f32, Color)>,
    },
}

pub struct Paint {
    kind: PaintKind,
    pub blend_mode: BlendMode,
    pub anti_alias: bool,
}

impl Paint {
    fn new(kind: PaintKind) -> Self {
        Self {
            kind,
            anti_alias: true,
            blend_mode: Default::default(),
        }
    }

    pub fn color(color: Color) -> Self {
        Self::new(PaintKind::Color(color))
    }

    pub fn linear_gradient(
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        stops: Vec<(f32, Color)>,
    ) -> Self {
        Self::new(PaintKind::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
            stops,
        })
    }

    pub fn radial_gradient(
        cx: f32,
        cy: f32,
        in_radius: f32,
        out_radius: f32,
        stops: Vec<(f32, Color)>,
    ) -> Self {
        Self::new(PaintKind::RadialGradient {
            cx,
            cy,
            in_radius,
            out_radius,
            stops,
        })
    }

    fn to_impl_paint(&self) -> femtovg::Paint {
        let mut paint = match &self.kind {
            PaintKind::Color(c) => femtovg::Paint::color((*c).into()),
            PaintKind::LinearGradient {
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
                stops.iter().map(|(t, c)| (*t, (*c).into())),
            ),
            PaintKind::RadialGradient {
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
                stops.iter().map(|(t, c)| (*t, (*c).into())),
            ),
        };
        paint.set_anti_alias(self.anti_alias);
        paint
    }
}

#[derive(Default)]
pub struct Stroke {
    pub width: f32,
    pub miter_limit: f32,
    pub line_cap_start: LineCap,
    pub line_cap_end: LineCap,
    pub line_join: LineJoin,
}

impl Stroke {
    fn apply_to_paint(&self, paint: &mut femtovg::Paint) {
        paint.set_line_width(self.width);
        paint.set_miter_limit(self.miter_limit);
        paint.set_line_cap_start(self.line_cap_start.into());
        paint.set_line_cap_end(self.line_cap_end.into());
        paint.set_line_join(self.line_join.into());
    }
}

pub struct Path(femtovg::Path);

impl Path {
    pub fn new() -> Self {
        Self(femtovg::Path::new())
    }

    pub fn move_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.0.move_to(x, y);
        self
    }

    pub fn line_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.0.line_to(x, y);
        self
    }

    pub fn bezier_to(
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

    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) -> &mut Self {
        self.0.quad_to(cx, cy, x, y);
        self
    }

    pub fn arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) -> &mut Self {
        self.0.arc_to(x1, y1, x2, y2, radius);
        self
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) -> &mut Self {
        self.0.rect(x, y, w, h);
        self
    }

    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32) -> &mut Self {
        self.0.ellipse(cx, cy, rx, ry);
        self
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32) -> &mut Self {
        self.0.circle(cx, cy, r);
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.0.close();
        self
    }
}
