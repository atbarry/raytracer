struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) uv: vec2<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) position: vec3<f32>,
  @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  let pos = model.position;
  out.clip_position = vec4<f32>(pos, 1.0);
  out.position = model.position;
  out.uv = vec2<f32>(model.uv.x, 1.0 - model.uv.y);
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(in.position.rgb, 1.0);
}

fn cbrt(f: f32) -> f32 {
  return pow(f, 1.0/3.0);
}

