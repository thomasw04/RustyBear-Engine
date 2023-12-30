use crate::{
    context::VisContext,
    render::types::{FragmentShader, VertexShader},
    utils::Guid,
};

pub struct Shader {
    module: wgpu::ShaderModule,
    stages: what::ShaderStages,
    guid: Guid,
}

impl Shader {
    pub fn new(
        context: &VisContext,
        guid: Guid,
        spirv: Vec<u32>,
        stages: what::ShaderStages,
    ) -> Result<Self, String> {
        let module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::SpirV(spirv.into()),
            });

        Ok(Self {
            module,
            stages,
            guid,
        })
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
