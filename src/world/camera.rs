use std::io::Write;

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec3Swizzles, Vec4};
use winit::keyboard::KeyCode;

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub pos: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub focal_length: f32,
    pub samples_per_pixel: u32,
    pub frames_to_render: u32,
    pub current_frame: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraRaw {
    pos: Vec4,
    forward: Vec4,
    right: Vec4,
    up: Vec4,
    focal_length: f32,
    samples_per_pixel: u32,
    frames_to_render: u32,
    current_frame: u32,
}

impl Camera {
    pub fn to_raw(&self) -> CameraRaw {
        CameraRaw {
            pos: self.pos.xyzz(),
            forward: self.forward.xyzz(),
            right: self.right.xyzz(),
            up: self.up.xyzz(),
            focal_length: self.focal_length,
            samples_per_pixel: self.samples_per_pixel,
            frames_to_render: self.frames_to_render,
            current_frame: self.current_frame,
        }
    }

    pub fn key_press(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::KeyK => self.samples_per_pixel *= 2,
            KeyCode::KeyJ => {
                self.samples_per_pixel /= 2;
                if self.samples_per_pixel == 0 {
                    self.samples_per_pixel = 1;
                }
            },
            KeyCode::KeyI => self.frames_to_render *= 2,
            KeyCode::KeyU => {
                self.frames_to_render /= 2;
                if self.frames_to_render == 0 {
                    self.frames_to_render = 1;
                }
            },
            _ => return false,
        }

        true
    }

    pub fn increase_frame(&mut self) {
        self.current_frame += 1;
        progress_bar((self.current_frame as f32) / (self.frames_to_render as f32));
    }

    pub fn to_string(&self) -> String {
        format!("Frames: {}, RaysPerPixel: {}", self.frames_to_render, self.samples_per_pixel)
    }

    pub fn reset_render(&mut self) {
        self.current_frame = 0;
        println!("\nRendering with:\n{}", self.to_string());
    }
}

fn progress_bar(mut progress: f32) {
    progress = progress.clamp(0.0, 1.0);
    let pieces = 100;
    let pieces_to_show = (progress * (pieces as f32)) as i32;

    let progress_str: String = (0..pieces_to_show).map(|_| '#').collect();
    let empty: String = (0..pieces - pieces_to_show).map(|_| ' ').collect();

    print!("Progress: {:.2}% [{}{}]\r", progress * 100.0, progress_str, empty);
    std::io::stdout().flush().unwrap();
}
