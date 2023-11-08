struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) tex_coord: vec2<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) pos: vec3<f32>,
  @location(1) tex_coord: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.tex_coord = vec2<f32>(in.tex_coord.x, 1.0 - in.tex_coord.y);
  out.clip_position = vec4<f32>(in.position, 1.0);
  out.pos = in.position;
  return out;
}

@group(0) @binding(0) var color_buffer: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let tex_coord = in.tex_coord;
  var color = textureSample(color_buffer, screen_sampler, tex_coord); 
  return color;
}

