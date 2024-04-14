use wgpu::{CommandEncoder, TextureView};

use super::framebuffer::Framebuffer;

pub fn create_color_renderpass<'a>(
    encoder: &'a mut CommandEncoder, view: &'a TextureView, fbo: &'a Framebuffer, clear: bool,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: match fbo.sample_count() {
                1 => view,
                _ => fbo.get_view(),
            },
            resolve_target: match fbo.sample_count() {
                1 => None,
                _ => Some(view),
            },
            ops: wgpu::Operations {
                load: if clear {
                    wgpu::LoadOp::Clear(wgpu::Color::BLUE)
                } else {
                    wgpu::LoadOp::Load
                },
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        ..Default::default()
    })
}
