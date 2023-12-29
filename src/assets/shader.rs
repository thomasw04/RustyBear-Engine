use crate::context::VisContext;

pub struct Shader {
    module: wgpu::ShaderModule,
    stages: what::ShaderStages,
}

impl Shader {
    pub fn new(
        context: &VisContext,
        spirv: Vec<u32>,
        stages: what::ShaderStages,
    ) -> Result<Self, String> {
        let module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::SpirV(spirv.into()),
            });

        Ok(Self { module, stages })
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }

    pub fn stages(&self) -> what::ShaderStages {
        self.stages
    }
}
