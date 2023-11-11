use std::{f32::consts::PI, io::Write};

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec3Swizzles, Vec4};
use winit::{
    event::{Modifiers, MouseButton, MouseScrollDelta},
    keyboard::{KeyCode, ModifiersKeyState, ModifiersState}, dpi::PhysicalPosition,
};

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub pos: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub z_near: f32,
    pub z_far: f32,
    pub fov: f32,
    pub resoultion: Vec2,
    pub samples_per_pixel: u32,
    pub frames_to_render: u32,
    pub current_frame: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraRaw {
    world_to_pixel: Mat4,
    pixel_to_world: Mat4,
    pos: Vec4,
    focal_length: f32,
    samples_per_pixel: u32,
    frames_to_render: u32,
    current_frame: u32,
}

impl Camera {
    pub fn to_raw(&self) -> CameraRaw {
        CameraRaw {
            pixel_to_world: self.calculate_world_to_pixel().inverse(),
            world_to_pixel: self.calculate_world_to_pixel(),
            pos: self.pos.xyzz(),
            focal_length: self.z_near,
            samples_per_pixel: self.samples_per_pixel,
            frames_to_render: self.frames_to_render,
            current_frame: self.current_frame,
        }
    }

    pub fn calculate_world_to_pixel(&self) -> Mat4 {
        let u = 1920.0;
        let v = 1080.0;
        let clip_to_pixel = Mat4::from_cols_array(&[
            u / 2.0,
            0.0,
            0.0,
            0.0,
            0.0,
            v / 2.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            u / 2.0,
            v / 2.0,
            0.0,
            1.0,
        ]);

        let n = self.z_near;
        let f = self.z_far;
        let t = (self.fov * PI / 2.0).tan();
        let a = self.resoultion.x / self.resoultion.y;

        let perspective_proj = Mat4::from_cols_array(&[
            1.0 / (a * t),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0 / t,
            0.0,
            0.0,
            0.0,
            0.0,
            -f / (f - n),
            -1.0,
            0.0,
            0.0,
            -f * n,
            0.0,
        ]);

        let x = self.pos.x;
        let y = self.pos.y;
        let z = self.pos.z;

        let world_to_camera = Mat4::from_cols_array(&[
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -x, -y, -z, 1.0,
        ]);

        return clip_to_pixel * perspective_proj * world_to_camera;
    }

    pub fn key_press(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::KeyK => self.samples_per_pixel *= 2,
            KeyCode::KeyJ => {
                self.samples_per_pixel /= 2;
                if self.samples_per_pixel == 0 {
                    self.samples_per_pixel = 1;
                }
            }
            KeyCode::KeyI => self.frames_to_render *= 2,
            KeyCode::KeyU => {
                self.frames_to_render /= 2;
                if self.frames_to_render == 0 {
                    self.frames_to_render = 1;
                }
            }
            _ => return false,
        }

        true
    }

    pub fn mouse_scroll(&mut self, delta: MouseScrollDelta, modifiers: &Modifiers) -> bool {
        if modifiers.state() != ModifiersState::CONTROL {
            return false;
        }

        let val = match delta {
            MouseScrollDelta::LineDelta(_y, x) => -x,
            MouseScrollDelta::PixelDelta(pos) => {
                (-pos.x / 5.0) as f32
            }
        };

        let old = (self.fov * PI / 2.0).tan().ln();
        let new = old + val / 5.0;
        self.fov = 2.0 * new.exp().atan() / PI;
        dbg!(self.fov);

        true
    }

    pub fn increase_frame(&mut self) {
        self.current_frame += 1;
        let msg = format!(
            "{}/{}",
            self.current_frame.clamp(0, self.frames_to_render),
            self.frames_to_render
        );
        progress_bar(
            (self.current_frame as f32) / (self.frames_to_render as f32),
            &msg,
        );
    }

    pub fn to_string(&self) -> String {
        format!(
            "Frames: {}, RaysPerPixel: {}",
            self.frames_to_render, self.samples_per_pixel
        )
    }

    pub fn reset_render(&mut self) {
        self.current_frame = 0;
        println!("\nRendering with:\n{}", self.to_string());
    }
}

fn progress_bar(mut progress: f32, msg: &str) {
    progress = progress.clamp(0.0, 1.0);
    let pieces = 100;
    let pieces_to_show = (progress * (pieces as f32)) as i32;

    let progress_str: String = (0..pieces_to_show).map(|_| '#').collect();
    let empty: String = (0..pieces - pieces_to_show).map(|_| ' ').collect();

    print!("Progress: {} [{}{}]\r", msg, progress_str, empty);
    std::io::stdout().flush().unwrap();
}
