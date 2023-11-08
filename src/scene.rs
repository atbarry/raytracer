use crate::render_env::RenderEnv;
use bytemuck::{bytes_of, Pod, Zeroable};
use glam::{Vec3, Vec3Swizzles, Vec4};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};

use self::objects::{ObjectData, Sphere};

mod objects;

pub struct Scene {
    camera: Camera,
    objects: ObjectData,
    camera_buffer: Buffer,
    objects_buffer: Buffer,
    frames_since_change: i32,
    frames_since_change_buffer: Buffer,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

impl Scene {
    pub fn new(render_env: &RenderEnv) -> Self {
        let device = &render_env.device;

        let objects = ObjectData {
            spheres: Sphere::random_bunch(),
        };

        let camera = Camera {
            pos: Vec3::new(0.0, 0.0, 0.0),
            forward: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            right: Vec3::new(1.0, 0.0, 0.0),
            focal_length: 2.0,
        };

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytes_of(&camera.to_raw()),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let objects_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: objects.as_bytes(),
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });

        let frames_since_change = 0;
        let frames_since_change_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytes_of(&frames_since_change),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Scene bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: ShaderStages::COMPUTE,
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: ShaderStages::COMPUTE,
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: ShaderStages::COMPUTE,
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scene bind group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: objects_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: frames_since_change_buffer.as_entire_binding(),
                },
            ],
        });

        return Self {
            camera,
            objects,
            frames_since_change,
            camera_buffer,
            objects_buffer,
            frames_since_change_buffer,
            bind_group_layout,
            bind_group,
        };
    }

    pub fn scene_was_updated(&mut self, render_env: &RenderEnv) {
        self.frames_since_change = 0;
        render_env.queue.write_buffer(
            &self.frames_since_change_buffer,
            0,
            bytes_of(&self.frames_since_change),
        );
    }

    pub fn increase_frame(&mut self, render_env: &RenderEnv) {
        self.frames_since_change += 1;
        dbg!(self.frames_since_change);
        render_env.queue.write_buffer(
            &self.frames_since_change_buffer,
            0,
            bytes_of(&self.frames_since_change),
        );
    }

    pub fn reload(&mut self, render_env: &RenderEnv) {
        *self = Self::new(render_env);
        self.scene_was_updated(render_env);
    }

    fn update_buffers(&mut self, queue: &Queue) {
        queue.write_buffer(&self.camera_buffer, 0, bytes_of(&self.camera.to_raw()));
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pos: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    focal_length: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraRaw {
    pos: Vec4,
    forward: Vec4,
    right: Vec4,
    up: Vec3,
    focal_length: f32,
}

impl Camera {
    fn to_raw(&self) -> CameraRaw {
        CameraRaw {
            pos: self.pos.xyzz(),
            forward: self.forward.xyzz(),
            right: self.right.xyzz(),
            up: self.up,
            focal_length: self.focal_length,
        }
    }
}
