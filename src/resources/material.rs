use bytemuck::{Pod, Zeroable, Contiguous};

#[derive(Debug, Clone, Copy)]
pub struct Material {
    ty: MaterialType,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct MaterialRaw {
    ty: u8
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum MaterialType {
    Metal = 0,
}

impl Material {
    fn to_raw(&self) -> MaterialRaw {
        MaterialRaw {
            ty: self.ty as u8,
        }
    }
}


