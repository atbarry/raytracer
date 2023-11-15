use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4, vec3, vec4, Vec3Swizzles};
use rand::Rng;

use super::Material;

pub struct ObjectData {
    pub spheres: Vec<Sphere>,
}

impl ObjectData {
    pub fn as_bytes<'a>(&'a self) -> &'a [u8] {
        bytemuck::cast_slice(&self.spheres)
    }
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Sphere {
    pub center: Vec4,
    pub radius: f32,
    pub material_index: u32,
    pub padding: u64,
}

impl Sphere {
    pub fn random_bunch(amount: u32, materials_len: u32) -> Vec<Sphere> {
        let from = vec3(-1.0, -1.0, -1.0) * (amount as f32).sqrt();
        let to = vec3(1.0, 1.0, -2.0) * (amount as f32).sqrt();

        let mut rng = rand::thread_rng();

        (0..amount).into_iter().map(|_|
            Sphere::new(
                rng.gen::<Vec3>() * (to - from) + from,
                1.0,
                rng.gen_range(0..materials_len + 4)
            )
        ).collect()
    }

    pub fn new(center: Vec3, radius: f32, material_index: u32) -> Self {
        Self {
            center: center.xyzz(), 
            radius,
            material_index,
            padding: 0,
        }
    }
}


