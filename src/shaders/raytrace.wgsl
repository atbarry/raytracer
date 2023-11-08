struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) uv: vec2<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) pos: vec3<f32>,
  @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
  let pos = model.position;

  var out: VertexOutput;
  out.clip_position = vec4<f32>(pos, 1.0);
  // out.pos = (pos + vec3<f32>(1.0, 1.0, 0.0)) * 0.5;
  out.pos = pos;
  out.uv = vec2<f32>(model.uv.x, 1.0 - model.uv.y);
  return out;
}

@group(0) @binding(0)
var<uniform> time: f32;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let x = rand(in.pos.x + in.pos.y + time);
  let y = rand(in.pos.x + in.pos.y + time + 0.1);
  let z = rand(in.pos.x + in.pos.y + time + 0.2);
  return vec4<f32>(x, y, z, 1.0);
}


fn random_noise(xy: vec2<f32>, seed: f32) -> f32 {
  return rand(xy.x + xy.y + seed);
}

// fn random_color(seed: f32) -> vec3<f32> {
// }

fn rand(seed: f32) -> f32 {
  let x = hash(bitcast<u32>(seed));
  return f32(x % 256u) / 255.0;
}

fn hash(value: u32) -> u32 {
  var x = value;
  x += ( x << 10u );
  x ^= ( x >>  6u );
  x += ( x <<  3u );
  x ^= ( x >> 11u );
  x += ( x << 15u );
  return x;
}

fn gold_noise(xy: vec2<f32>, seed: f32) -> f32 {
  let PHI = 1.61803398874989484820459;  // Φ = Golden Ratio   
  return floor(tan(distance(xy*PHI, xy)*seed)*xy.x);
}

///
/// Color Stuff from this point on
///
struct Oklch { L: f32, C: f32, h: f32 }

fn oklab_to_oklch(L: f32, a: f32, b: f32) -> Oklch {
  let C = sqrt(pow(a,2.0) + pow(b, 2.0));
  let h = atan2(b,a);
  return Oklch(L, C, h);
}

fn srgb_to_oklch(c: vec3<f32>) -> Oklch {
  let l = 0.4122214708f * c.r + 0.5363325363f * c.g + 0.0514459929f * c.b;
  let m = 0.2119034982f * c.r + 0.6806995451f * c.g + 0.1073969566f * c.b;
  let s = 0.0883024619f * c.r + 0.2817188376f * c.g + 0.6299787005f * c.b;

  let l_ = cbrt(l);
  let m_ = cbrt(m);
  let s_ = cbrt(s);

  // in lab format
  let L = 0.2104542553f*l_ + 0.7936177850f*m_ - 0.0040720468f*s_;
  let a = 1.9779984951f*l_ - 2.4285922050f*m_ + 0.4505937099f*s_;
  let b = 0.0259040371f*l_ + 0.7827717662f*m_ - 0.8086757660f*s_;

  return oklab_to_oklch(L,a,b);
}

fn cbrt(f: f32) -> f32 {
  return pow(f, 1.0/3.0);
}

fn oklch_to_srgb(c: Oklch) -> vec3<f32> {
  // Convert to OkLab first
  let a = c.C * cos(c.h);
  let b = c.C * sin(c.h);
  let L = c.L;

  let l_ = L + 0.3963377774f * a + 0.2158037573f * b;
  let m_ = L - 0.1055613458f * a - 0.0638541728f * b;
  let s_ = L - 0.0894841775f * a - 1.2914855480f * b;

  let l = l_*l_*l_;
  let m = m_*m_*m_;
  let s = s_*s_*s_;

  return vec3<f32>(
     4.0767416621f * l - 3.3077115913f * m + 0.2309699292f * s,
    -1.2684380046f * l + 2.6097574011f * m - 0.3413193965f * s,
    -0.0041960863f * l - 0.7034186147f * m + 1.7076147010f * s,
  );
}

fn oklch_to_srgba(c: Oklch) -> vec4<f32> {
  let c_srgb = oklch_to_srgb(c);
  return vec4<f32>(c_srgb.xyz, 1.0);
}
