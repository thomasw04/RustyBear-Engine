use std::sync::Arc;

use crate::{
    context::VisContext,
    render::types::{FragmentShader, VertexShader},
    utils::Guid,
};

//Convience enum for handling assets that contain both a vertex and fragment shader or just one of them.
pub enum ShaderVariant {
    Single(Arc<Shader>),
    Double(Arc<Shader>, Arc<Shader>),
}

impl ShaderVariant {
    pub fn vertex_module(&self) -> &wgpu::ShaderModule {
        match self {
            ShaderVariant::Single(shader) => shader.module(),
            ShaderVariant::Double(shader, _) => shader.module(),
        }
    }

    pub fn fragment_module(&self) -> &wgpu::ShaderModule {
        match self {
            ShaderVariant::Single(shader) => shader.module(),
            ShaderVariant::Double(_, shader) => shader.module(),
        }
    }

    pub fn vertex_guid(&self) -> Guid {
        match self {
            ShaderVariant::Single(shader) => VertexShader::guid(shader.as_ref()),
            ShaderVariant::Double(shader, _) => VertexShader::guid(shader.as_ref()),
        }
    }

    pub fn fragment_guid(&self) -> Guid {
        match self {
            ShaderVariant::Single(shader) => FragmentShader::guid(shader.as_ref()),
            ShaderVariant::Double(_, shader) => FragmentShader::guid(shader.as_ref()),
        }
    }
}

pub struct Shader {
    module: wgpu::ShaderModule,
    stages: what::ShaderStages,
    guid: Guid,
}

impl Shader {
    pub fn new(
        context: &VisContext, guid: Guid, source: wgpu::ShaderSource, stages: what::ShaderStages,
    ) -> Result<Self, String> {
        let module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source });

        Ok(Self { module, stages, guid })
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }

    pub fn stages(&self) -> what::ShaderStages {
        self.stages
    }
}

impl VertexShader for Shader {
    fn guid(&self) -> Guid {
        self.guid
    }

    fn module(&self) -> &wgpu::ShaderModule {
        //TODO maybe check if the shader is a vertex shader?
        &self.module
    }
}

impl FragmentShader for Shader {
    fn guid(&self) -> Guid {
        self.guid
    }

    fn module(&self) -> &wgpu::ShaderModule {
        //TODO maybe check if the shader is a fragment shader?
        &self.module
    }
}
