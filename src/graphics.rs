//! Graphics support.

use std::sync::Arc;

use crate::{image::AsImgRef, math};
use canvasette::Renderer;
pub use canvasette::{font, Canvas, Drawable, PreparedText, Texture, TextureSlice};
pub use imgref::ImgRef;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

/// An 8-bit RGBA color.
pub type Color = rgb::Rgba<u8>;

/// Encapsulates graphics device and rendering state.
pub struct Graphics {
    window: Arc<winit::window::Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter: wgpu::Adapter,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    canvasette_renderer: Renderer,
}

/// A texture that can be rendered to.
///
/// Framebuffers may be created via [`Graphics::create_framebuffer`].
pub struct Framebuffer(wgpu::Texture);

impl Framebuffer {
    /// Gets the underlying texture as a [`TextureSlice`], which may be used for sprite drawing.
    pub fn as_texture_slice(&self) -> TextureSlice {
        TextureSlice::from(&self.0)
    }
}

async fn new_wgpu_instance() -> wgpu::Instance {
    // Taken from https://github.com/emilk/egui/blob/454abf705b87aba70cef582d6ce80f74aa398906/crates/eframe/src/web/web_painter_wgpu.rs#L117-L166
    //
    // We try to see if we can use default backends first to initialize an adapter. If not, we fall back on GL.
    let instance = wgpu::Instance::default();

    if instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            ..Default::default()
        })
        .await
        .is_none()
    {
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        })
    } else {
        instance
    }
}

impl Graphics {
    pub(crate) async fn new(window: winit::window::Window) -> Self {
        let window = Arc::new(window);

        let instance = new_wgpu_instance().await;
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    required_features: wgpu::Features::default(),
                    ..Default::default()
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let mut surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface_config.present_mode = wgpu::PresentMode::AutoVsync;
        surface.configure(&device, &surface_config);

        let canvasette_renderer =
            Renderer::new(&device, surface.get_capabilities(&adapter).formats[0]);

        window.request_redraw();

        Self {
            window,
            device,
            queue,
            adapter,
            surface,
            surface_config,
            canvasette_renderer,
        }
    }

    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.width = size.width.max(1);
        self.surface_config.height = size.height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
        self.window.request_redraw();
    }

    pub(crate) fn get_current_frame(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface.get_capabilities(&self.adapter).formats[0]
    }

    /// Adds a font.
    pub fn add_font(&mut self, font: &[u8]) -> Vec<font::Attrs> {
        self.canvasette_renderer.add_font(font)
    }

    /// Prepares text for rendering.
    pub fn prepare_text(
        &mut self,
        contents: impl AsRef<str>,
        metrics: font::Metrics,
        attrs: font::Attrs,
    ) -> PreparedText {
        self.canvasette_renderer
            .prepare_text(contents, metrics, attrs)
    }

    /// Retrieve the underlying window.
    pub fn window(&self) -> Window {
        Window(&self.window)
    }

    /// Creates an empty framebuffer texture.
    pub fn create_framebuffer(&self, size: math::UVec2) -> Framebuffer {
        Framebuffer(self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("teenygame: Framebuffer"),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        }))
    }

    /// Loads a texture.
    pub fn load_texture<'a>(&self, img: impl AsImgRef<Color>) -> Texture {
        let (buf, width, height) = img.as_ref().to_contiguous_buf();

        Texture::from(self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: Some("teenygame: Texture"),
                size: wgpu::Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::default(),
            &bytemuck::cast_slice(&buf),
        ))
    }

    fn render_to_texture(&mut self, canvas: &Canvas, texture: &wgpu::Texture) {
        self.canvasette_renderer
            .prepare(&self.device, &self.queue, texture.size(), canvas)
            .unwrap();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("teenygame: encoder"),
            });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            self.canvasette_renderer.render(&mut rpass);
        }

        self.queue.submit(Some(encoder.finish()));
    }

    /// Renders to a framebuffer.
    pub fn render_to_framebuffer(&mut self, canvas: &Canvas, framebuffer: &Framebuffer) {
        self.render_to_texture(canvas, &framebuffer.0);
    }

    pub(crate) fn render(&mut self, canvas: &Canvas) {
        let frame = self
            .get_current_frame()
            .expect("failed to acquire next swap chain texture");
        self.render_to_texture(&canvas, &frame.texture);
        self.window.pre_present_notify();
        frame.present();
        self.window.request_redraw();
    }
}

/// Window.
pub struct Window<'a>(&'a winit::window::Window);

impl<'a> Window<'a> {
    /// Sets the title of the window.
    pub fn set_title(&self, title: &str) {
        self.0.set_title(title);
    }

    /// Requests the size of the window to be a given size.
    pub fn set_size(&self, size: math::UVec2, resizable: bool) {
        self.0.set_resizable(resizable);
        let _ = self.0.request_inner_size(PhysicalSize::new(size.x, size.y));
    }

    /// Gets the current size of the window.
    pub fn size(&self) -> math::UVec2 {
        let size = self.0.inner_size();
        math::UVec2::new(size.width, size.height)
    }

    /// Gets the scale factor of the window.
    pub fn scale_factor(&self) -> f64 {
        self.0.scale_factor()
    }
}
