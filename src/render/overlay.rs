use crate::context::VisContext;

use super::{factory::PipelineFactory, framebuffer::Framebuffer};

pub struct RendererGUI {
    pipelines: PipelineFactory,
}

pub fn render_overlay(ctx: &VisContext, encoder: &mut wgpu::CommandEncoder) {}
