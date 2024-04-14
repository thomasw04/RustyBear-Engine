use crate::{
    context::VisContext,
    render::types::{FragmentShader, MaterialLayout, VertexShader},
    utils::{Guid, TypeDisplay},
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use super::{
    assets::{Assets, Ptr},
    types::AssetError,
};

//Convience enum for handling assets that contain both a vertex and fragment shader or just one of them.
pub enum ShaderVariant<'a> {
    Single(&'a Shader),
    Double(&'a Shader, &'a Shader),
}

impl<'a> ShaderVariant<'a> {
    pub fn build_single(ptr: Ptr<Shader>, assets: &'a Assets) -> Result<Self, AssetError> {
        let shader = assets.try_get(&ptr)?;
        Ok(ShaderVariant::Single(shader))
    }

    pub fn build_double(
        vertex: Ptr<Shader>, fragment: Ptr<Shader>, assets: &'a Assets,
    ) -> Result<Self, AssetError> {
        if vertex == fragment {
            return Self::build_single(vertex, assets);
        }

        let vertex = assets.try_get(&vertex)?;
        let fragment = assets.try_get(&fragment)?;
        Ok(ShaderVariant::Double(vertex, fragment))
    }

    pub fn from_material(
        material: &impl MaterialLayout, assets: &'a Assets,
    ) -> Result<Self, AssetError> {
        let vptr = VertexShader::ptr(material);
        let fptr = FragmentShader::ptr(material);

        if vptr == fptr {
            return Self::build_single(*vptr, assets);
        }

        let vertex = assets.try_get(vptr)?;
        let fragment = assets.try_get(fptr)?;
        Ok(ShaderVariant::Double(vertex, fragment))
    }

    pub fn vertex(&self) -> &Shader {
        match self {
            ShaderVariant::Single(shader) => shader,
            ShaderVariant::Double(shader, _) => shader,
        }
    }

    pub fn fragment(&self) -> &Shader {
        match self {
            ShaderVariant::Single(shader) => shader,
            ShaderVariant::Double(_, shader) => shader,
        }
    }

    pub fn vertex_id(&self) -> Ptr<Shader> {
        match self {
            ShaderVariant::Single(shader) => Ptr::new(shader.guid),
            ShaderVariant::Double(shader, _) => Ptr::new(shader.guid),
        }
    }

    pub fn fragment_id(&self) -> Ptr<Shader> {
        match self {
            ShaderVariant::Single(shader) => Ptr::new(shader.guid),
            ShaderVariant::Double(_, shader) => Ptr::new(shader.guid),
        }
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        match self {
            ShaderVariant::Single(shader) => {
                shader.guid.hash(&mut hasher);
                hasher.finish()
            }
            ShaderVariant::Double(vertex, fragment) => {
                vertex.guid.hash(&mut hasher);
                fragment.guid.hash(&mut hasher);
                hasher.finish()
            }
        }
    }
}

impl<'a> From<&'a Shader> for ShaderVariant<'a> {
    fn from(shader: &'a Shader) -> Self {
        ShaderVariant::Single(shader)
    }
}

pub struct Shader {
    module: wgpu::ShaderModule,
    stages: what::ShaderStages,
    guid: Guid,
}

impl TypeDisplay for Shader {
    fn type_name() -> &'static str {
        "Shader"
    }
}

#[profiling::all_functions]
impl Shader {
    pub fn new(
        context: &VisContext, guid: Guid, source: wgpu::ShaderSource, stages: what::ShaderStages,
    ) -> Result<Self, String> {
        let module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source });

        Ok(Self { module, stages, guid })
    }

    pub fn change_guid(&mut self, guid: Guid) {
        self.guid = guid;
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }

    pub fn stages(&self) -> what::ShaderStages {
        self.stages
    }
}
