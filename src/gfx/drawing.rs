use std::{
    collections::HashMap,
    ffi::c_void,
    ops::{Mul, MulAssign},
    sync::{Arc, Weak},
};

use femtovg::{renderer::OpenGl, ImageFlags, ImageId, Paint, PixelFormat, Transform2D};

use crate::asset::Image;

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

pub trait Drawable {
    fn get_image_id(&self, canvas: &mut Canvas) -> femtovg::ImageId;
    fn size(&self, canvas: &mut Canvas) -> (u32, u32);
}

impl Drawable for Arc<Image> {
    fn get_image_id(&self, canvas: &mut Canvas) -> femtovg::ImageId {
        canvas.get_or_create_image(self.clone(), ImageFlags::NEAREST)
    }

    fn size(&self, _canvas: &mut Canvas) -> (u32, u32) {
        self.as_ref().size()
    }
}

pub struct Framebuffer(ImageId);

impl Drawable for Arc<Framebuffer> {
    fn get_image_id(&self, _canvas: &mut Canvas) -> femtovg::ImageId {
        self.0
    }

    fn size(&self, canvas: &mut Canvas) -> (u32, u32) {
        let size = canvas.canvas.image_info(self.0).unwrap().size();
        (size.width as u32, size.height as u32)
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

pub struct Canvas {
    canvas: femtovg::Canvas<OpenGl>,
    size: (u32, u32),
    framebuffer: Option<Arc<Framebuffer>>,
    image_id_cache: HashMap<(*const c_void, ImageFlags), CachedImageId>,
}

impl Canvas {
    pub(super) fn new(canvas: femtovg::Canvas<OpenGl>) -> Self {
        Self {
            canvas,
            size: (0, 0),
            framebuffer: None,
            image_id_cache: HashMap::new(),
        }
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    fn get_or_create_image(&mut self, img: Arc<Image>, flags: ImageFlags) -> femtovg::ImageId {
        match self
            .image_id_cache
            .entry((img.as_ref() as *const _ as *const c_void, flags))
        {
            std::collections::hash_map::Entry::Occupied(e) => e.get().id,
            std::collections::hash_map::Entry::Vacant(e) => {
                let id = self
                    .canvas
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

    pub(crate) fn gc(&mut self) {
        self.image_id_cache.retain(|_, c| {
            if c.weak.strong_count() > 0 {
                return true;
            }

            self.canvas.delete_image(c.id);
            false
        });
    }

    pub(super) fn set_size(&mut self, width: u32, height: u32, dpi: f32) {
        self.size = (width, height);
        self.canvas.set_size(width, height, dpi);
    }

    pub(super) fn flush(&mut self) {
        self.canvas.flush();
    }

    pub fn create_framebuffer(&mut self, width: u32, height: u32) -> Arc<Framebuffer> {
        let flags = ImageFlags::FLIP_Y | ImageFlags::NEAREST;
        let id = self
            .canvas
            .create_image_empty(width as usize, height as usize, PixelFormat::Rgba8, flags)
            .unwrap();
        let fb = Arc::new(Framebuffer(id));
        self.image_id_cache.insert(
            (fb.as_ref() as *const _ as *const c_void, flags),
            CachedImageId {
                weak: Box::new(Arc::downgrade(&fb)),
                id,
            },
        );
        fb
    }

    pub fn set_framebuffer(&mut self, fb: Option<Arc<Framebuffer>>) {
        if let Some(fb) = &fb {
            self.canvas
                .set_render_target(femtovg::RenderTarget::Image(fb.0));
        } else {
            self.canvas.set_render_target(femtovg::RenderTarget::Screen);
        }
        self.framebuffer = fb;
    }

    pub fn push_state(&mut self) {
        self.canvas.save();
    }

    pub fn transform(&mut self, t: &AffineTransform) {
        self.canvas.set_transform(&Transform2D(t.0.clone()));
    }

    pub fn pop_state(&mut self) {
        self.canvas.restore();
    }

    #[inline]
    pub fn draw_image<D>(&mut self, d: &D, x: f32, y: f32)
    where
        D: Drawable,
    {
        let (iw, ih) = d.size(self);
        self.draw_image_destination_scale(d, x, y, iw as f32, ih as f32);
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
        D: Drawable,
    {
        let (iw, ih) = d.size(self);
        self.draw_image_source_clip_destination_scale(
            d, 0.0, 0.0, iw as f32, ih as f32, x, y, width, height,
        );
    }

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
        D: Drawable,
    {
        let (iw, ih) = d.size(self);
        let id = d.get_image_id(self);
        self.canvas.fill_path(
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
        self.canvas.clear_rect(x, y, width, height, color.into());
    }

    pub fn fill_path(&mut self, path: &Path, fill: &Fill) {
        self.canvas.fill_path(&path.0, &fill.to_paint());
    }

    pub fn stroke_path(&mut self, path: &Path, stroke: &Stroke, fill: &Fill) {
        let mut paint = fill.to_paint();
        stroke.apply_to_paint(&mut paint);
        self.canvas.stroke_path(&path.0, &fill.to_paint());
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
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
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

enum FillKind {
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

pub struct Fill {
    kind: FillKind,
    pub anti_alias: bool,
}

impl Fill {
    pub fn color(color: Color) -> Self {
        Self {
            kind: FillKind::Color(color),
            anti_alias: true,
        }
    }

    pub fn linear_gradient(
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        stops: Vec<(f32, Color)>,
    ) -> Self {
        Self {
            kind: FillKind::LinearGradient {
                start_x,
                start_y,
                end_x,
                end_y,
                stops,
            },
            anti_alias: true,
        }
    }

    pub fn radial_gradient(
        cx: f32,
        cy: f32,
        in_radius: f32,
        out_radius: f32,
        stops: Vec<(f32, Color)>,
    ) -> Self {
        Self {
            kind: FillKind::RadialGradient {
                cx,
                cy,
                in_radius,
                out_radius,
                stops,
            },
            anti_alias: true,
        }
    }

    fn to_paint(&self) -> femtovg::Paint {
        let mut paint = match &self.kind {
            FillKind::Color(c) => Paint::color((*c).into()),
            FillKind::LinearGradient {
                start_x,
                start_y,
                end_x,
                end_y,
                stops,
            } => Paint::linear_gradient_stops(
                *start_x,
                *start_y,
                *end_x,
                *end_y,
                stops.iter().map(|(t, c)| (*t, (*c).into())),
            ),
            FillKind::RadialGradient {
                cx,
                cy,
                in_radius,
                out_radius,
                stops,
            } => Paint::radial_gradient_stops(
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
