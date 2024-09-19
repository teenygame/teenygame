pub mod drawing;
pub mod ninepatch;

use std::{rc::Rc, sync::Arc};

pub use drawing::Canvas;
use drawing::Framebuffer;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    window::Window,
};

pub(crate) struct GraphicsState {
    pub window: Rc<Window>,
    pub canvas: drawing::Canvas,

    #[cfg(not(target_arch = "wasm32"))]
    gl: Option<Gl>,
}

#[cfg(not(target_arch = "wasm32"))]
struct Gl {
    context: glutin::context::PossiblyCurrentContext,
    surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

pub struct GraphicsContext<'a> {
    canvas: &'a mut drawing::Canvas,
}

impl<'a> GraphicsContext<'a> {
    pub(crate) fn new(canvas: &'a mut drawing::Canvas) -> Self {
        Self { canvas }
    }

    pub fn create_framebuffer(&mut self, width: u32, height: u32) -> Arc<Framebuffer> {
        self.canvas.create_framebuffer(width, height)
    }
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

            let renderer = femtovg::renderer::OpenGl::new_from_html_canvas(&canvas).unwrap();

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

            let renderer = unsafe {
                femtovg::renderer::OpenGl::new_from_function_cstr(|s| {
                    gl_display.get_proc_address(s) as *const _
                })
            }
            .unwrap();

            (
                window,
                renderer,
                Gl {
                    context: gl_context,
                    surface: gl_surface,
                },
            )
        };

        let mut canvas = Canvas::new(femtovg::Canvas::new(renderer).unwrap());

        let dpi = window.scale_factor();
        let size = window.inner_size();
        canvas.set_size(size.width, size.height, dpi as f32);

        window.request_redraw();

        Self {
            window: Rc::new(window),
            canvas,
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

        {
            let dpi = self.window.scale_factor();
            self.canvas.set_size(size.width, size.height, dpi as f32);
        }
    }

    pub fn flush_and_swap_buffers(&mut self) {
        self.canvas.flush();

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
