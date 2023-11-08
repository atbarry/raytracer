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
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Sphere {
    color: Vec4,
    center: Vec3,
    radius: f32,
}

impl Sphere {
    pub fn random_bunch() -> Vec<Sphere> {
        let from = vec3(-2.0, -1.0, -5.0);
        let to = vec3(2.0, 1.0, -20.0);

        let mut rng = rand::thread_rng();
        let num = rng.gen_range(5..22);

        (0..num).into_iter().map(|_| 
            Sphere {
                color: rng.gen(),
                center: rng.gen::<Vec3>() * (to - from) + from,
                radius: rng.gen_range(0.2..1.5),
            }
        ).collect()
    }
}

