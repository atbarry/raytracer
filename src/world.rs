use std::collections::HashSet;
use crate::{common::UNIFORM_BUFFER_BINDING, render_env::RenderEnv, resources::Material};
use bytemuck::{bytes_of, Pod, Zeroable, cast_slice};
use glam::{vec2, vec3, Vec3, Vec3Swizzles, Vec4, Vec2};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Modifiers, MouseButton, MouseScrollDelta},
    keyboard::KeyCode,
};

use crate::resources::{
    Camera,
    ObjectData,
    Sphere,
};

pub struct World {
    objects: ObjectData,
    objects_buffer: Buffer,
    materials: Vec<Material>,
    materials_buffer: Buffer,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

impl World {
    pub fn new(render_env: &RenderEnv, camera_buffer: &Buffer) -> Self {
        let device = &render_env.device;

        let materials = vec![Material::random_new(), Material::random_new()];
        let spheres = vec![
            Sphere::new(vec3(0.0, -100.5, 0.0), 100.0, 0),
            Sphere::new(vec3(0.0, 0.0, -1.0), -2.0, 0),
        ];
        let objects = ObjectData {
            spheres: Sphere::random_bunch(20, materials.len() as u32),
        };
        dbg!(&materials, spheres);

        let objects_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: objects.as_bytes(),
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });

        let materials_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: cast_slice(&materials.as_slice()),
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
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
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
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
                    resource: materials_buffer.as_entire_binding(),
                },
            ],
        });

        return Self {
            objects,
            objects_buffer,
            materials,
            materials_buffer,
            bind_group_layout,
            bind_group,
        };
    }

    pub fn reload(&mut self, render_env: &RenderEnv, camera_buffer: &Buffer) {
        *self = Self::new(render_env, camera_buffer);
    }
}
