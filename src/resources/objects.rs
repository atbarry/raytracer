use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4, vec3};
use rand::Rng;

pub struct ObjectData {
    pub spheres: Vec<Sphere>,
}

impl ObjectData {
    pub fn as_bytes<'a>(&'a self) -> &'a [u8] {
        bytemuck::cast_slice(&self.spheres)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Sphere {
    pub color: Vec4,
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn random_bunch() -> Vec<Sphere> {
        let from = vec3(-1.0, -1.0, -1.0) * 4.0;
        let to = vec3(1.0, 1.0, -2.0) * 4.0;

        let mut rng = rand::thread_rng();
        let num = rng.gen_range(30..45);

        (0..num).into_iter().map(|_| 
            Sphere {
                color: rng.gen(),
                center: rng.gen::<Vec3>() * (to - from) + from,
                radius: rng.gen::<f32>().powi(2) * 2.0 + 0.2,
            }
        ).collect()
    }

    pub fn new(center: Vec3, radius: f32) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            center,
            radius,
            color: rng.gen(),
        }
    }
}

