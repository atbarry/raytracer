use bytemuck::{Pod, Zeroable, Contiguous};
use glam::{Vec3, vec3, Vec4};
use rand::{distributions::{self, Standard}, prelude::Distribution, rngs::ThreadRng, Rng, thread_rng};

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct Material {
    pub color: Vec4,
    pub metallic: f32,
    pub roughness: f32,
    pub specular: f32,
    pub padding: f32,
}


#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum MaterialType {
    Metal = 0,
}

impl Material {
    pub fn random_new() -> Material {
        thread_rng().gen()
    }
}

impl Distribution<Material> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Material {
        Material {
            color: rng.gen(),
            metallic: rng.gen(),
            roughness: rng.gen(),
            specular: rng.gen(),
            padding: 0.0,
        }
    }
}
