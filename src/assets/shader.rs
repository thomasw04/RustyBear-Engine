use crate::context::VisContext;

use super::assets::Ptr;

//Convience enum for handling assets that contain both a vertex and fragment shader or just one of them.
pub enum ShaderVariant {
    Single(Ptr<Shader>),
    Double(Ptr<Shader>, Ptr<Shader>),
}

impl ShaderVariant {
    pub fn vertex(&self) -> &Ptr<Shader> {
        match self {
            ShaderVariant::Single(shader) => shader,
            ShaderVariant::Double(shader, _) => shader,
        }
    }

    pub fn fragment(&self) -> &Ptr<Shader> {
        match self {
            ShaderVariant::Single(shader) => shader,
            ShaderVariant::Double(_, shader) => shader,
        }
    }
}

pub struct Shader {
    module: wgpu::ShaderModule,
    stages: what::ShaderStages,
}

impl Shader {
    pub fn new(
        context: &VisContext, source: wgpu::ShaderSource, stages: what::ShaderStages,
    ) -> Result<Self, String> {
        let module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source });

        Ok(Self { module, stages })
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }

    pub fn stages(&self) -> what::ShaderStages {
        self.stages
    }
}
