use std::{f32::consts::PI, io::Write, collections::HashSet};

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec3Swizzles, Vec4, vec2, vec4, vec3};
use winit::{
    event::{Modifiers, MouseButton, MouseScrollDelta, ElementState},
    keyboard::{KeyCode, ModifiersKeyState, ModifiersState}, dpi::PhysicalPosition,
};

use crate::render_env::RenderEnv;

const RIGHT: Vec3 = vec3(1.0, 0.0, 0.0);
const UP: Vec3 = vec3(0.0, 1.0, 0.0);
const FORWARD: Vec3 = vec3(0.0, 0.0, -1.0);

#[derive(Copy, Clone, Debug)]
struct Drag {
    pub last_mouse_pos: Vec2,
}

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub pos: Vec3,
    z_near: f32,
    z_far: f32,
    fov: f32,
    resolution: Vec2, 
    samples_per_pixel: u32,
    frames_to_render: u32,
    current_frame: u32,
    speed: f32,
    drag: Option<Drag>,
    look_direction: Vec3,
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
    pub fn new(render_env: &RenderEnv, pos: Vec3) -> Camera {
        let res = render_env.window.inner_size();
        Camera {
            pos,
            z_near: 1.0,
            z_far: 100000.0,
            resolution: vec2(res.width as f32, res.height as f32),
            fov: 0.25,
            samples_per_pixel: 1,
            frames_to_render: 8,
            current_frame: 0,
            drag: None,
            look_direction: vec3(0.0, 0.0, -1.0),
            speed: 0.05,
        }
    }
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

    fn clip_space_to_pixel(&self) -> Mat4 {
        let u = self.resolution.x;
        let v = self.resolution.y;
        Mat4::from_cols_array(&[
            u / 2.0, 0.0, 0.0, 0.0,
            0.0, v / 2.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0, u / 2.0,
            v / 2.0, 0.0, 1.0,
        ])
    }

    fn perspective_proj(&self) -> Mat4 {
        let n = self.z_near;
        let f = self.z_far;
        let t = (self.fov * PI / 2.0).tan();
        let a = self.resolution.x / self.resolution.y;

        Mat4::from_cols_array(&[
            1.0 / (a * t), 0.0, 0.0, 0.0,
            0.0, 1.0 / t, 0.0, 0.0,
            0.0, 0.0, -f / (f - n), -1.0,
            0.0, 0.0, -f * n, 0.0,
        ])
    }

    fn rotation_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(vec3(0.0,0.0,0.0), self.look_direction, UP)
    }

    pub fn calculate_world_to_pixel(&self) -> Mat4 {
        let x = self.pos.x;
        let y = self.pos.y;
        let z = self.pos.z;

        let world_to_camera_pos = Mat4::from_cols_array(&[
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            -x, -y, -z, 1.0,
        ]);

        self.clip_space_to_pixel() * self.perspective_proj() * self.rotation_matrix() * world_to_camera_pos
    }

    pub fn key_press(&mut self, key: KeyCode, keys_held: &HashSet<KeyCode>) -> bool {
        let mut changed = true;
        match key {
            KeyCode::KeyK => self.samples_per_pixel = (self.samples_per_pixel * 2).min(256),
            KeyCode::KeyJ => self.samples_per_pixel = (self.samples_per_pixel / 2).max(1),
            KeyCode::KeyI => self.frames_to_render = (self.frames_to_render * 2).min(256),
            KeyCode::KeyU => self.frames_to_render = (self.frames_to_render / 2).max(1),
            KeyCode::ArrowUp => self.zoom(-1.0),
            KeyCode::ArrowDown => self.zoom(1.0),
            KeyCode::Backspace => {
                self.frames_to_render = 1;
                self.samples_per_pixel = 1
            }
            _ => changed = false,
        }

        let inverse_rotation = self.rotation_matrix().inverse();
        let mut held = |code: KeyCode| {
            let c = keys_held.contains(&code);
            if c { changed = true; }
            c
        };
        let get_translation = |v: Vec3| {
            let p = inverse_rotation.transform_point3(v);
            p.xyz().normalize() * self.speed
        };
        let mut move_dir_on_key = |key: KeyCode, dir: Vec3| {
            if held(key) {
                self.pos += get_translation(dir);
            }
        };

        move_dir_on_key(KeyCode::KeyW, FORWARD);
        move_dir_on_key(KeyCode::KeyS, -FORWARD);
        move_dir_on_key(KeyCode::KeyD, RIGHT);
        move_dir_on_key(KeyCode::KeyA, -RIGHT);
        move_dir_on_key(KeyCode::Space, UP);
        move_dir_on_key(KeyCode::ShiftLeft, -UP);
        changed
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

        self.zoom(val);
        true
    }

    fn zoom(&mut self, val: f32) {
        let old = (self.fov * PI / 2.0).tan().ln();
        let new = old + val / 5.0;
        self.fov = 2.0 * new.exp().atan() / PI;
    }

    // makes the camera look at a certain pixel based on its offset from the center
    fn look_at_pixel_from_center(&mut self, offset: Vec2) {
        dbg!(self.resolution);
        let pixel = self.resolution / 2.0 + offset;
        let offset_3d = vec3(pixel.x, pixel.y, 0.0);
        let pixel_to_world = self.calculate_world_to_pixel().inverse();
        self.look_direction = pixel_to_world.transform_point3(offset_3d).normalize();
    }

    pub fn mouse_drag(&mut self,
        render_env: &RenderEnv,
        mouse_pos: Vec2,
        state: Option<ElementState>,
        button: Option<MouseButton>,
    ) -> bool {
        if Some(MouseButton::Left) == button {
            let state = state.unwrap();

            if !state.is_pressed() && self.drag.is_some() {
                render_env.window.set_cursor_visible(true);
                self.drag = None;
                return false;
            }

            if self.drag.is_none() && state.is_pressed() {
                render_env.window.set_cursor_visible(false);
                self.drag = Some(Drag { last_mouse_pos: mouse_pos });
            }

            return true;
        }

        let Some(drag) = &mut self.drag else {
            return false;
        };

        let delta = drag.last_mouse_pos - mouse_pos;
        drag.last_mouse_pos = self.resolution / 2.0;
        let center = PhysicalPosition::<f64>::new(drag.last_mouse_pos.x as f64, drag.last_mouse_pos.y as f64);
        if let Err(e) = render_env.window.set_cursor_position(center) {
            dbg!("Unable to set cursor pos", e);
        };
        self.look_at_pixel_from_center(delta);
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
