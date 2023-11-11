struct Camera {
  world_to_pixel: mat4x4<f32>,
  pixel_to_world: mat4x4<f32>,
  focal_length: f32,
  samples_per_pixel: u32,
  frames_to_render: u32,
  current_frame: u32,
}


// Vectors
const ZERO: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
const ONE: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
const UP: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0);
const RIGHT: vec3<f32> = vec3<f32>(1.0, 0.0, 0.0);
const FORWARD: vec3<f32> = vec3<f32>(0.0, 0.0, -1.0);

var<private> uv: vec2<u32>; 
var<private> size: vec2<u32>;
@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;
@group(2) @binding(0) var<uniform> camera: Camera;

@compute @workgroup_size(1,1,1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  uv = global_invocation_id.xy;
  size = textureDimensions(color_buffer);
  let pixel_in_world = projection(vec4<f32> (f32(uv.x), f32(size.y - uv.y), 1.0, 1.0));
  write_color(proj.xyz);
}

fn projection(value: vec4<f32>) -> vec4<f32> {
  var v = (camera.pixel_to_world * value);
  v /= v.w;
  return v;
}

fn write_color(color: vec3<f32>) {
  textureStore(color_buffer, uv, to_vec4(color));
}

fn to_vec4(x: vec3<f32>) -> vec4<f32> {
  return vec4<f32> (x.xyz, 1.0);
}
