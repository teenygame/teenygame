pub mod ninepatch;

use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Weak},
};

use winit::{
    dpi::{LogicalSize, PhysicalSize},
    window::Window,
};

#[cfg(feature = "femtovg")]
pub use femtovg::*;

use crate::asset::Image;

#[cfg(feature = "femtovg")]
pub type Canvas = femtovg::Canvas<femtovg::renderer::OpenGl>;

#[cfg(feature = "femtovg")]
pub trait CanvasExt {
    fn draw_image(&mut self, id: ImageId, x: f32, y: f32);

    fn draw_image_destination_scale(
        &mut self,
        id: ImageId,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    );

    fn draw_image_source_clip_destination_scale(
        &mut self,
        id: ImageId,
        sx: f32,
        sy: f32,
        s_width: f32,
        s_height: f32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    );
}

#[cfg(feature = "femtovg")]
impl CanvasExt for Canvas {
    #[inline]
    fn draw_image(&mut self, id: ImageId, x: f32, y: f32) {
        let (iw, ih) = self.image_size(id).unwrap();
        self.draw_image_source_clip_destination_scale(
            id, 0.0, 0.0, iw as f32, ih as f32, x, y, iw as f32, ih as f32,
        );
    }

    #[inline]
    fn draw_image_destination_scale(
        &mut self,
        id: ImageId,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let (iw, ih) = self.image_size(id).unwrap();
        self.draw_image_source_clip_destination_scale(
            id, 0.0, 0.0, iw as f32, ih as f32, x, y, width, height,
        );
    }

    fn draw_image_source_clip_destination_scale(
        &mut self,
        id: ImageId,
        sx: f32,
        sy: f32,
        s_width: f32,
        s_height: f32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let (iw, ih) = self.image_size(id).unwrap();
        self.fill_path(
            &PathBuilder::new().rect(x, y, width, height).build(),
            &Paint::image(
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
}

pub struct GraphicsContext {
    #[cfg(feature = "femtovg")]
    pub canvas: Canvas,

    #[cfg(feature = "femtovg")]
    image_id_cache: HashMap<(*const Image, ImageFlags), CachedImageId>,
}

struct CachedImageId {
    weak: Weak<Image>,
    id: ImageId,
}

impl GraphicsContext {
    #[cfg(feature = "femtovg")]
    pub fn get_or_create_image(
        &mut self,
        img: Arc<Image>,
        flags: ImageFlags,
    ) -> Result<femtovg::ImageId, femtovg::ErrorKind> {
        match self
            .image_id_cache
            .entry((img.as_ref() as *const Image, flags))
        {
            std::collections::hash_map::Entry::Occupied(e) => Ok(e.get().id),
            std::collections::hash_map::Entry::Vacant(e) => {
                let id = self.canvas.create_image(
                    femtovg::ImageSource::try_from(&*img)
                        .map_err(|_| femtovg::ErrorKind::UnsupportedImageFormat)?,
                    flags,
                )?;
                e.insert(CachedImageId {
                    weak: Arc::downgrade(&img),
                    id,
                });
                Ok(id)
            }
        }
    }

    pub(super) fn gc(&mut self) {
        #[cfg(feature = "femtovg")]
        {
            self.image_id_cache.retain(|_, c| {
                if c.weak.strong_count() > 0 {
                    return true;
                }

                self.canvas.delete_image(c.id);
                false
            });
        }
    }
}

pub(crate) struct GraphicsState {
    pub window: Rc<Window>,
    pub context: GraphicsContext,

    #[cfg(not(target_arch = "wasm32"))]
    gl: Option<Gl>,
}

#[cfg(not(target_arch = "wasm32"))]
struct Gl {
    context: glutin::context::PossiblyCurrentContext,
    surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

#[cfg(not(target_arch = "wasm32"))]
fn create_gl_context(
    window: &Window,
    gl_config: &glutin::config::Config,
) -> glutin::context::NotCurrentContext {
    use glutin::display::GetGlDisplay;
    use glutin::prelude::*;
    use raw_window_handle::HasWindowHandle;

    let raw_window_handle = window.window_handle().ok().map(|wh| wh.as_raw());

    let context_attributes =
        glutin::context::ContextAttributesBuilder::new().build(raw_window_handle);

    let fallback_context_attributes = glutin::context::ContextAttributesBuilder::new()
        .with_context_api(glutin::context::ContextApi::Gles(None))
        .build(raw_window_handle);

    let legacy_context_attributes = glutin::context::ContextAttributesBuilder::new()
        .with_context_api(glutin::context::ContextApi::OpenGl(Some(
            glutin::context::Version::new(2, 1),
        )))
        .build(raw_window_handle);

    let gl_display = gl_config.display();

    unsafe {
        gl_display
            .create_context(gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(gl_config, &fallback_context_attributes)
                    .unwrap_or_else(|_| {
                        gl_display
                            .create_context(gl_config, &legacy_context_attributes)
                            .expect("failed to create context")
                    })
            })
    }
}

impl GraphicsState {
    #[allow(unused_variables, unused_mut)]
    pub fn new(mut gfx: Option<Self>, event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let mut window_attrs = Window::default_attributes().with_title("");

        #[cfg(target_arch = "wasm32")]
        let (window, renderer) = {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();

            #[cfg(feature = "femtovg")]
            let renderer = femtovg::renderer::OpenGl::new_from_html_canvas(&canvas).unwrap();

            #[cfg(not(feature = "femtovg"))]
            let renderer = ();

            window_attrs = window_attrs.with_canvas(Some(canvas));

            let window = event_loop
                .create_window(window_attrs)
                .expect("failed to create window");

            (window, renderer)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (window, renderer, gl_graphics) = {
            use glutin::config::GetGlConfig;
            use glutin::display::GetGlDisplay;
            use glutin::prelude::*;
            use glutin_winit::GlWindow;

            let (window, gl_context) = if let Some(mut gfx) = gfx.take() {
                let gl = gfx.gl.take().unwrap();
                (
                    glutin_winit::finalize_window(event_loop, window_attrs, &gl.context.config())
                        .unwrap(),
                    gl.context,
                )
            } else {
                let display_builder =
                    glutin_winit::DisplayBuilder::new().with_window_attributes(Some(window_attrs));
                let (window, gl_config) = display_builder
                    .build(
                        event_loop,
                        glutin::config::ConfigTemplateBuilder::new().with_alpha_size(8),
                        |mut configs| configs.next().unwrap(),
                    )
                    .unwrap();
                let window = window.unwrap();

                let gl_context = create_gl_context(&window, &gl_config).treat_as_possibly_current();

                (window, gl_context)
            };

            let _ = window.request_inner_size(LogicalSize::new(1280, 720));

            let gl_config = gl_context.config();
            let gl_display = gl_config.display();

            let attrs = window
                .build_surface_attributes(glutin::surface::SurfaceAttributesBuilder::new())
                .expect("Failed to build surface attributes");
            let gl_surface = unsafe {
                gl_display
                    .create_window_surface(&gl_config, &attrs)
                    .unwrap()
            };

            gl_context.make_current(&gl_surface).unwrap();

            #[cfg(feature = "femtovg")]
            let renderer = unsafe {
                femtovg::renderer::OpenGl::new_from_function_cstr(|s| {
                    gl_display.get_proc_address(s) as *const _
                })
            }
            .unwrap();

            #[cfg(not(feature = "femtovg"))]
            let renderer = ();

            (
                window,
                renderer,
                Gl {
                    context: gl_context,
                    surface: gl_surface,
                },
            )
        };

        #[cfg(feature = "femtovg")]
        let canvas = {
            let mut canvas = Canvas::new(renderer).unwrap();
            let dpi = window.scale_factor();
            let size = window.inner_size();
            canvas.set_size(size.width, size.height, dpi as f32);
            canvas
        };

        window.request_redraw();

        Self {
            window: Rc::new(window),

            context: GraphicsContext {
                #[cfg(feature = "femtovg")]
                canvas,

                #[cfg(feature = "femtovg")]
                image_id_cache: HashMap::new(),
            },

            #[cfg(not(target_arch = "wasm32"))]
            gl: Some(gl_graphics),
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            let canvas = self.window.canvas().unwrap();
            canvas.set_width(size.width);
            canvas.set_height(size.height);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            use glutin::surface::GlSurface;
            let Gl { context, surface } = self.gl.as_ref().unwrap();
            surface.resize(
                context,
                std::num::NonZero::new(size.width).unwrap(),
                std::num::NonZero::new(size.height).unwrap(),
            );
        }

        #[cfg(feature = "femtovg")]
        {
            let dpi = self.window.scale_factor();
            self.context
                .canvas
                .set_size(size.width, size.height, dpi as f32);
        }
    }

    pub fn flush_and_swap_buffers(&mut self) {
        #[cfg(feature = "femtovg")]
        self.context.canvas.flush();

        #[cfg(not(target_arch = "wasm32"))]
        {
            use glutin::surface::GlSurface;
            let Gl { context, surface } = self.gl.as_ref().unwrap();
            surface.swap_buffers(context).unwrap();
        }
    }

    pub fn suspend(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use glutin::prelude::{NotCurrentGlContext, PossiblyCurrentGlContext};
            let Gl { context, surface } = self.gl.take().unwrap();
            self.gl = Some(Gl {
                context: context
                    .make_not_current()
                    .unwrap()
                    .treat_as_possibly_current(),
                surface,
            });
        }
    }
}

#[cfg(feature = "femtovg")]
pub struct PathBuilder(femtovg::Path);

#[cfg(feature = "femtovg")]
impl PathBuilder {
    pub fn new() -> Self {
        Self(femtovg::Path::new())
    }

    pub fn move_to(mut self, x: f32, y: f32) -> Self {
        self.0.move_to(x, y);
        self
    }

    pub fn line_to(mut self, x: f32, y: f32) -> Self {
        self.0.line_to(x, y);
        self
    }

    pub fn bezier_to(mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) -> Self {
        self.0.bezier_to(c1x, c1y, c2x, c2y, x, y);
        self
    }

    pub fn quad_to(mut self, cx: f32, cy: f32, x: f32, y: f32) -> Self {
        self.0.quad_to(cx, cy, x, y);
        self
    }

    pub fn close(mut self) -> Self {
        self.0.close();
        self
    }

    pub fn solidity(mut self, solidity: Solidity) -> Self {
        self.0.solidity(solidity);
        self
    }

    pub fn arc(mut self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, dir: Solidity) -> Self {
        self.0.arc(cx, cy, r, a0, a1, dir);
        self
    }

    pub fn arc_to(mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) -> Self {
        self.0.arc_to(x1, y1, x2, y2, radius);
        self
    }

    pub fn rect(mut self, x: f32, y: f32, w: f32, h: f32) -> Self {
        self.0.rect(x, y, w, h);
        self
    }

    pub fn rounded_rect(mut self, x: f32, y: f32, w: f32, h: f32, r: f32) -> Self {
        self.0.rounded_rect(x, y, w, h, r);
        self
    }

    pub fn rounded_rect_varying(
        mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rad_top_left: f32,
        rad_top_right: f32,
        rad_bottom_right: f32,
        rad_bottom_left: f32,
    ) -> Self {
        self.0.rounded_rect_varying(
            x,
            y,
            w,
            h,
            rad_top_left,
            rad_top_right,
            rad_bottom_right,
            rad_bottom_left,
        );
        self
    }

    pub fn ellipse(mut self, cx: f32, cy: f32, rx: f32, ry: f32) -> Self {
        self.0.ellipse(cx, cy, rx, ry);
        self
    }

    pub fn circle(mut self, cx: f32, cy: f32, r: f32) -> Self {
        self.0.circle(cx, cy, r);
        self
    }

    pub fn build(self) -> femtovg::Path {
        self.0
    }
}
