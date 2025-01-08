//! Graphics support.

pub use canvasette::{font, Canvas, Color, Drawable, Image, Label, TextureSlice};

pub(crate) fn render_to_texture(
    wgpu: &wginit::Wgpu,
    canvasette_renderer: &mut canvasette::Renderer,
    fonts: &mut FontLibrary,
    canvas: &Canvas,
    texture: &wgpu::Texture,
) {
    canvasette_renderer
        .prepare(
            &wgpu.device,
            &wgpu.queue,
            &mut fonts.0,
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

/// Font library for storing fonts.
pub struct FontLibrary(cosmic_text::FontSystem);

impl FontLibrary {
    pub(crate) fn new() -> Self {
        Self(cosmic_text::FontSystem::new_with_locale_and_db(
            sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string()),
            cosmic_text::fontdb::Database::new(),
        ))
    }

    /// Adds a new font to the library, returning all faces in that font as [`font::Attrs`].
    pub fn add_font(&mut self, font: &[u8]) -> Vec<font::Attrs> {
        self.0
            .db_mut()
            .load_font_source(cosmic_text::fontdb::Source::Binary(std::sync::Arc::new(
                font.to_vec(),
            )))
            .into_iter()
            .flat_map(|id| self.0.db().face(id))
            .flat_map(|face_info| {
                face_info.families.iter().map(|(name, _)| font::Attrs {
                    family: font::Family::Name(name.clone()),
                    stretch: face_info.stretch,
                    style: face_info.style,
                    weight: face_info.weight,
                })
            })
            .collect::<Vec<_>>()
    }

    /// Creates a label for rendering.
    pub fn create_label(
        &mut self,
        contents: impl AsRef<str>,
        metrics: font::Metrics,
        attrs: font::Attrs,
    ) -> Label {
        Label::new(&mut self.0, contents.as_ref(), metrics, attrs)
    }
}
