//! Graphics support.

use crate::math;
pub use canvasette::{font, Canvas, Color, Drawable, Image, PreparedText, TextureSlice};
use winit::dpi::PhysicalSize;

pub(crate) fn render_to_texture(
    wgpu: &wginit::Wgpu,
    canvasette_renderer: &mut canvasette::Renderer,
    canvasette_cache: &mut canvasette::Cache,
    font_system: &mut cosmic_text::FontSystem,
    canvas: &Canvas,
    texture: &wgpu::Texture,
) {
    canvasette_renderer
        .prepare(
            &wgpu.device,
            &wgpu.queue,
            canvasette_cache,
            font_system,
            texture.size(),
            canvas,
        )
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

pub struct Graphics<'a> {
    pub(crate) canvasette_renderer: &'a mut canvasette::Renderer,
    pub(crate) font_system: &'a mut cosmic_text::FontSystem,
    pub(crate) window: &'a winit::window::Window,
}

impl<'a> Graphics<'a> {
    /// Adds a font.
    pub fn add_font(&mut self, font: &[u8]) {
        self.font_system.db_mut().load_font_data(font.to_vec());
    }

    /// Prepares text for rendering.
    pub fn prepare_text(
        &mut self,
        contents: impl AsRef<str>,
        metrics: font::Metrics,
        attrs: font::Attrs,
    ) -> PreparedText {
        self.canvasette_renderer
            .prepare_text(self.font_system, contents, metrics, attrs)
    }

    /// Retrieve the underlying window.
    pub fn window(&self) -> Window {
        Window(&self.window)
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
