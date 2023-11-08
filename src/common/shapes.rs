#![allow(unused)]
use wgpu::{util::DeviceExt, Buffer, BufferUsages, RenderPass};
use crate::render_env::RenderEnv;
use super::Vertex;

pub struct Shape {
    vertices: Vec<Vertex>,
    indices: Option<Vec<u32>>,
}

impl Shape {
    pub fn unit_square() -> Self {
        Self {
            vertices: vec![
                Vertex {
                    pos: [-1.0, -1.0, 0.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    pos: [1.0, -1.0, 0.0],
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    pos: [1.0, 1.0, 0.0],
                    tex_coords: [1.0, 1.0],
                },
                Vertex {
                    pos: [-1.0, 1.0, 0.0],
                    tex_coords: [0.0, 1.0],
                },
            ],
            indices: Some(vec![0, 1, 3, 1, 2, 3]),
        }
    }

    pub fn new(vertices: Vec<Vertex>, indices: Option<Vec<u32>>) -> Self {
        Self { vertices, indices }
    }
}

#[derive(Debug)]
pub struct Triangles {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_num: u32,
    vertex_num: u32,
}

impl Triangles {
    pub fn new(render_env: &RenderEnv, shapes: &[Shape]) -> Self {
        let mut combined_vertices = Vec::new();
        let mut combined_indices: Vec<u32> = Vec::new();
        let mut current_index = 0;

        for Shape {
            vertices,
            indices: possible_indicies,
        } in shapes
        {
            combined_vertices.extend_from_slice(vertices);
            let end_index = vertices.len() as u32 + current_index;
            if let Some(indices) = possible_indicies {
                combined_indices.extend(indices.iter().map(|i| i + current_index));
            } else {
                combined_indices.extend(current_index..end_index);
            }

            current_index = end_index;
        }

        let vertex_buffer = render_env
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&combined_vertices),
                usage: BufferUsages::VERTEX,
            });

        let index_buffer = render_env
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&combined_indices),
                usage: BufferUsages::INDEX,
            });

        let index_num = combined_indices.len() as u32;
        let vertex_num = combined_vertices.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            vertex_num,
            index_num,
        }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_num, 0, 0..1);
    }
}
