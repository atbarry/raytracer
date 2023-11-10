use crate::{render_env::RenderEnv, common::UNIFORM_BUFFER_BINDING};
use bytemuck::{bytes_of, Pod, Zeroable};
use glam::{Vec3, Vec3Swizzles, Vec4, vec3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};
use winit::keyboard::KeyCode;

use self::{objects::{ObjectData, Sphere}, camera::Camera};

mod objects;
mod camera;

pub struct World {
    camera: Camera,
    objects: ObjectData,
    camera_buffer: Buffer,
    objects_buffer: Buffer,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

impl World {
    pub fn new(render_env: &RenderEnv) -> Self {
        let device = &render_env.device;

        let spheres = vec![
            Sphere::new(vec3(0.0,-0.0, -1.0), 0.5),
            Sphere {
                color: Vec4::ONE,
                radius: 100.0,
                center: vec3(0.0, -100.5, -1.0),
            },
        ];
        let objects = ObjectData {
            spheres,
        };

        let camera = Camera {
            pos: Vec3::new(0.0, 0.0, 0.0),
            forward: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            right: Vec3::new(1.0, 0.0, 0.0),
            focal_length: 1.0,
            samples_per_pixel: 1,
            frames_to_render: 8,
            current_frame: 0,
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
            label: Some("Frames Since Change Buffer"),
            contents: bytes_of(&frames_since_change),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Scene bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    ty: UNIFORM_BUFFER_BINDING,
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
                    ty: UNIFORM_BUFFER_BINDING,
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
            camera_buffer,
            objects_buffer,
            bind_group_layout,
            bind_group,
        };
    }

    pub fn on_key_press(&mut self, render_env: &RenderEnv, key: KeyCode) {
        let queue = &render_env.queue;
        let mut scene_was_changed = false;
        if self.camera.key_press(key) {
            queue.write_buffer(&self.camera_buffer, 0, bytes_of(&self.camera.to_raw()));
            scene_was_changed = true;
        }

        if scene_was_changed {
            self.scene_was_updated(&render_env);
        }
    }

    fn scene_was_updated(&mut self, render_env: &RenderEnv) {
        self.camera.reset_render();
        render_env.queue.write_buffer(&self.camera_buffer, 0, bytes_of(&self.camera.to_raw()));
    }

    pub fn increase_frame(&mut self, render_env: &RenderEnv) {
        self.camera.increase_frame();
        render_env.queue.write_buffer(&self.camera_buffer, 0, bytes_of(&self.camera.to_raw()));
    }

    pub fn reload(&mut self, render_env: &RenderEnv) {
        let current_camera = self.camera;
        *self = Self::new(render_env);
        self.camera = current_camera.clone();
        self.scene_was_updated(render_env);
    }

    fn update_buffers(&mut self, queue: &Queue) {
        queue.write_buffer(&self.camera_buffer, 0, bytes_of(&self.camera.to_raw()));
    }
}

