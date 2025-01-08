//! Graphics support.

pub use canvasette::{font, Canvas, Color, Drawable, Image, Label, TextureSlice};

pub(crate) fn render_to_texture(
    wgpu: &wginit::Wgpu,
    canvasette_renderer: &mut canvasette::Renderer,
    font_system: &mut cosmic_text::FontSystem,
    canvas: &Canvas,
    texture: &wgpu::Texture,
) {
    canvasette_renderer
        .prepare(
            &wgpu.device,
            &wgpu.queue,
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
