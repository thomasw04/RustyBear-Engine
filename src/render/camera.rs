use std::f32::consts::PI;

use glam::{Mat4, Vec3, Vec4};
use wgpu::util::DeviceExt;

use crate::{
    context::{Context, VisContext},
    event::{self, EventSubscriber},
};

use super::types::CameraUniform;

pub struct PerspectiveCamera {
    position: Vec3,
    rotation: Vec3,
    fovy: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
    view: Mat4,
    projection: Mat4,
    dirty: bool,
}

impl EventSubscriber for PerspectiveCamera {
    fn on_event(&mut self, event: &crate::event::Event, _context: &mut Context) -> bool {
        match event {
            event::Event::Resized { width, height } => {
                self.aspect_ratio = *width as f32 / *height as f32;
                self.dirty = true;
                false
            }
            _ => false,
        }
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU: glam::Mat4 = glam::mat4
(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 0.5),
    Vec4::new(0.0, 0.0, 0.0, 1.5),
);

impl Default for PerspectiveCamera {
    fn default() -> Self {
        PerspectiveCamera {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
            fovy: 45.0,
            aspect_ratio: 1280.0 / 720.0,
            near: 0.1,
            far: 100.0,
            view: glam::Mat4::IDENTITY,
            projection: glam::Mat4::IDENTITY,
            dirty: true,
        }
    }
}

impl PerspectiveCamera {
    pub fn view_projection(&mut self) -> Mat4 {
        if self.dirty {
            self.calc_view_projection();
        }

        OPENGL_TO_WGPU * self.projection * self.view
    }

    fn calc_view_projection(&mut self) {
        self.set_projection(self.fovy, self.aspect_ratio, self.near, self.far);
        self.set_view(self.position, self.rotation);
        self.dirty = false;
    }

    pub fn set_projection(&mut self, fovy: f32, aspect_ratio: f32, near: f32, far: f32) {
        self.projection = glam::Mat4::perspective_rh(fovy * 180.0 / PI, aspect_ratio, near, far)
    }

    pub fn set_view(&mut self, position: Vec3, rotation: Vec3) {
        self.view = glam::Mat4::from_translation(position)
            * glam::Mat4::from_rotation_x(rotation.x * PI / 180.0)
            * glam::Mat4::from_rotation_y(rotation.y * PI / 180.0)
            * glam::Mat4::from_rotation_z(rotation.z * PI / 180.0);

        self.view = self.view.inverse();
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.dirty = true;
    }

    pub fn rotation(&self) -> Vec3 {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: Vec3) {
        self.rotation = rotation;
        self.dirty = true;
    }

    pub fn fovy(&self) -> f32 {
        self.fovy
    }

    pub fn set_fovy(&mut self, fovy: f32) {
        self.fovy = fovy;
        self.dirty = true;
    }

    pub fn far(&self) -> f32 {
        self.far
    }

    pub fn set_far(&mut self, far: f32) {
        self.far = far;
        self.dirty = true;
    }

    pub fn near(&self) -> f32 {
        self.near
    }

    pub fn set_near(&mut self, near: f32) {
        self.near = near;
        self.dirty = true;
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.dirty = true;
    }

    pub fn inc_pos(&mut self, size: Vec3) {
        self.position += size;
        self.dirty = true;
    }

    pub fn inc_rot(&mut self, size: Vec3) {
        self.rotation += size;
        self.dirty = true;
    }
}

pub struct CameraBuffer {
    name: String,
    bind_group: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
    camera_buffer: wgpu::Buffer,
    uniform: CameraUniform,
}

impl CameraBuffer {
    pub fn new(context: &VisContext, name: &str) -> CameraBuffer {
        let uniform = CameraUniform::default();
        let camera_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(name),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let layout = CameraBuffer::create_layout(context, name);

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(name),
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            });

        CameraBuffer {
            name: String::from(name),
            bind_group,
            layout,
            camera_buffer,
            uniform,
        }
    }

    fn create_layout(context: &VisContext, name: &str) -> wgpu::BindGroupLayout {
        context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(name),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            })
    }

    //TODO: Use some kind of staging buffer, for performance
    pub fn update_buffer(&mut self, context: &VisContext, camera: [[f32; 4]; 4]) {
        self.uniform.view_projection = camera;
        context.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
}
