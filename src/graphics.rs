//! Graphics support.

use crate::{image::AsImgRef, math};
pub use canvasette::{font, Canvas, Drawable, PreparedText, Texture, TextureSlice};
pub use imgref::ImgRef;
use wgpu::util::DeviceExt as _;
use winit::dpi::PhysicalSize;

/// An 8-bit RGBA color.
pub type Color = rgb::Rgba<u8>;

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

#[derive(PartialEq, Eq)]
struct DeviceId(*const wgpu::Device);

pub struct Graphics<'a> {
    pub(crate) canvasette_renderer: &'a mut canvasette::Renderer,
    pub(crate) wgpu: &'a wginit::Wgpu,
    pub(crate) window: &'a winit::window::Window,
}

pub(crate) fn render_to_texture(
    wgpu: &wginit::Wgpu,
    canvasette_renderer: &mut canvasette::Renderer,
    canvas: &Canvas,
    texture: &wgpu::Texture,
) {
    canvasette_renderer
        .prepare(&wgpu.device, &wgpu.queue, texture.size(), canvas)
        .unwrap();

    let mut encoder = wgpu
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
        canvasette_renderer.render(&mut rpass);
    }

    wgpu.queue.submit(Some(encoder.finish()));
}

impl<'a> Graphics<'a> {
    fn device_id(&self) -> DeviceId {
        DeviceId(&self.wgpu.device as *const _)
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
        Framebuffer(
            self.wgpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("teenygame: Framebuffer"),
                size: wgpu::Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self
                    .wgpu
                    .surface
                    .get_capabilities(&self.wgpu.adapter)
                    .formats[0],
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            }),
        )
    }

    /// Loads a texture.
    pub fn load_texture(&self, img: impl AsImgRef<Color>) -> Texture {
        let (buf, width, height) = img.as_ref().to_contiguous_buf();

        Texture::from(self.wgpu.device.create_texture_with_data(
            &self.wgpu.queue,
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

    /// Renders to a framebuffer.
    pub fn render_to_framebuffer(&mut self, canvas: &Canvas, framebuffer: &Framebuffer) {
        render_to_texture(
            &self.wgpu,
            &mut self.canvasette_renderer,
            canvas,
            &framebuffer.0,
        );
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

/// A lazily loaded resource.
pub struct Lazy<Resource>
where
    Resource: LazyLoadable,
{
    raw: Resource::Raw,
    loaded: Option<LazyLoaded<Resource>>,
}

struct LazyLoaded<Resource> {
    ready: Resource,
    device_id: DeviceId,
}

/// A resource that can be lazily loaded.
pub trait LazyLoadable {
    /// The raw resource.
    type Raw;

    /// Loads a raw resource into the graphics state and returns the loaded resource.
    fn load(graphics: &mut Graphics, raw: &Self::Raw) -> Self;
}

impl LazyLoadable for canvasette::Texture {
    type Raw = crate::image::Img<Vec<rgb::RGBA8>>;

    fn load(graphics: &mut Graphics, raw: &Self::Raw) -> Self {
        graphics.load_texture(raw.as_ref())
    }
}

impl LazyLoadable for Vec<font::Attrs> {
    type Raw = Vec<u8>;

    fn load(graphics: &mut Graphics, raw: &Self::Raw) -> Self {
        graphics.add_font(raw)
    }
}

impl<Resource> Lazy<Resource>
where
    Resource: LazyLoadable,
{
    /// Creates a lazy resource from the raw resource.
    pub fn new(raw: Resource::Raw) -> Self {
        Self { raw, loaded: None }
    }

    /// Gets the loaded resource, or loads it if not already loaded.
    ///
    /// When this function is first called, the resource will be loaded into the graphics state.
    ///
    /// If the graphics device is invalidated, the underlying resource will also be invalidated and a subsequent call to this function will reload it if a new graphics state is provided.
    pub fn get_or_load(&mut self, graphics: &mut Graphics) -> &Resource {
        let device_id = graphics.device_id();
        if let Some(loaded) = &self.loaded {
            if device_id != loaded.device_id {
                self.unload();
            }
        }

        &self
            .loaded
            .get_or_insert_with(|| LazyLoaded {
                ready: Resource::load(graphics, &self.raw),
                device_id,
            })
            .ready
    }

    /// Unloads the resource.
    pub fn unload(&mut self) {
        self.loaded = None;
    }
}
