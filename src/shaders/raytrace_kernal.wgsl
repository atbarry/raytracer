struct Sphere {
  center: vec3<f32>,
  padding: u32,
  radius: f32,
  material_index: u32,
}

struct Material {
  color: vec4<f32>,
  metallic: f32,
  specular: f32, 
  roughness: f32,
}

struct Ray {
  origin: vec3<f32>,
  direction: vec3<f32>,
}

struct HitRecord {
  point: vec3<f32>,
  normal: vec3<f32>,
  t: f32,
  index: u32,
  front_face: bool,
  hit: bool,
  material_index: u32,
}

struct Camera {
  world_to_pixel: mat4x4<f32>,
  pixel_to_world: mat4x4<f32>,
  pos: vec3<f32>,
  padding: u32,
  focal_length: f32,
  samples_per_pixel: u32,
  frames_to_render: u32,
  current_frame: u32,
}

struct ObjectData {
  spheres: array<Sphere>,
}

// Vectors
const ZERO: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
const ONE: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
const UP: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0);
const RIGHT: vec3<f32> = vec3<f32>(1.0, 0.0, 0.0);
const FORWARD: vec3<f32> = vec3<f32>(0.0, 0.0, -1.0);

// Shader variables
const RAY_TMAX: f32 = 10000000.0;
const RAY_TMIN: f32 = 0.001;
const MAX_RAY_DEPTH: i32 = 10;
const CACHE_ON: bool = true;

// Other constants
const PI: f32 = 3.1415926535897932385;
const MAX_U32: u32 = 4294967295u;

var<private> uv: vec2<u32>;
var<private> size: vec2<u32>;

@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;
@group(1) @binding(0) var color_cache: texture_2d<f32>;
@group(2) @binding(0) var<uniform> camera: Camera;
@group(2) @binding(1) var<storage, read> objects: ObjectData;
@group(2) @binding(2) var<storage, read> materials: array<Material>;
@group(3) @binding(0) var<uniform> time: f32;

@compute @workgroup_size(16,16,1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  size = textureDimensions(color_buffer);
  uv = global_invocation_id.xy;
  if camera.current_frame >= camera.frames_to_render && CACHE_ON {
    return;
  }

  initialize_rng();
  let pixel_color = send_rays();
  store_color(to_vec4(pixel_color));
}

fn pixel_to_world(pixel_uv: vec2<u32>) -> vec3<f32> {
  let uv4 = vec4<f32>(f32(pixel_uv.x), f32(size.y - pixel_uv.y), 0.0, 1.0);
  return (camera.pixel_to_world * uv4).xyz;
}

fn get_pixel_delta() -> vec3<f32> {
  let center = pixel_to_world(uv);
  let uvnext = pixel_to_world(vec2<u32>(1u,1u) + uv);
  return uvnext - center;
}

fn store_color(pixel_color: vec4<f32>) {
  var color = pixel_color;
  if CACHE_ON {
    let cached_color = textureLoad(color_cache, uv, 0);
    color = combine_pixel_cache_color(pixel_color, cached_color);
  }

  textureStore(color_buffer, uv, color);
}

fn combine_pixel_cache_color(pixel_color: vec4<f32>, cached_color: vec4<f32>) -> vec4<f32> {
  let frame = camera.current_frame;
  let next_frame = f32(frame + 1u);
  var cw = f32(frame) / next_frame;
  let rw = 1.0 - cw;
  return pixel_color*rw + cached_color*cw; 
}

fn send_rays() -> vec3<f32> {
  let pixel_center = pixel_to_world(uv);
  let pixel_delta = get_pixel_delta();
  var color = ZERO;
  for(var sample = 0u; sample < camera.samples_per_pixel; sample++) {
    let ray = get_random_ray(pixel_center, pixel_delta, sample);
    color += ray_color(ray);
  }
  return color / f32(camera.samples_per_pixel);
}

fn ray_color(start_ray: Ray) -> vec3<f32> {
  var rec: HitRecord;
  var ray = start_ray;
  var color: vec3<f32> = ZERO;
  var reflectance = 1.0;
  var depth: i32;
  for (depth = 0; depth <= MAX_RAY_DEPTH; depth++) {
    rec = trace(ray);
    if !rec.hit {
      color += miss(ray) * reflectance;
      break;
    } else {
      color += scatter(&ray, rec, &reflectance);
    }
  }

  return color;
}

fn miss(ray: Ray) -> vec3<f32> {
  let a = 0.5 * (1.0 - ray.direction.y);
  let color = (1.0 - a) * ONE + a * vec3(0.5, 0.7, 1.0);
  return color;
}

fn scatter(ray: ptr<function, Ray>, rec: HitRecord, reflectance: ptr<function, f32>) -> vec3<f32> {
  let material = materials[rec.material_index];
  var color: vec3<f32> = material.color.xyz;
  let local_reflectance = 0.90;
  color *= (1.0 - local_reflectance) * (*reflectance);
  *reflectance *= local_reflectance;

  let b = sin(4.0 * rec.normal.x * PI);
  let c = cos(4.0 * rec.normal.y * PI);
  let d = cos(4.0 * rec.normal.y * PI) + sin(4.0 * rec.normal.x * PI);

  (*ray).origin = rec.point;
  if rec.index % 2u == 0u {
    (*ray).direction = reflection((*ray).direction, rec.normal);
  } else {
    (*ray).direction = rec.normal + random_unit_vector();
  }
  return color;
}

fn reflection(dir: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
  return dir - 2.0 * dot(dir, normal) * normal;
}

fn get_random_ray(pixel_center: vec3<f32>, pixel_delta: vec3<f32>, ray_index: u32) -> Ray {
  let px = -0.5 + rand();
  let py = -0.5 + rand();
  let pixel_sample = pixel_center + vec3<f32>(px, py, 1.0) * pixel_delta;

  return Ray(camera.pos, pixel_sample);
}

fn trace(ray: Ray) -> HitRecord {
  var rec: HitRecord;
  var temp_rec: HitRecord;
  var closest_so_far = RAY_TMAX;

  for (var i = 0u; i < arrayLength(&objects.spheres); i++) {
    let sphere = objects.spheres[i];
    if hit_sphere(ray, sphere, RAY_TMIN, closest_so_far, &temp_rec) {
      closest_so_far = temp_rec.t;
      rec = temp_rec;
      rec.index = i;
      rec.hit = true;
    }
  }

  return rec;
}

fn hit_sphere(
  ray: Ray, 
  sphere: Sphere, 
  ray_tmin: f32, 
  ray_tmax: f32, 
  rec: ptr<function, HitRecord>
) -> bool {
  let oc = ray.origin - sphere.center;
  let a = length_squared(ray.direction);
  let half_b = dot(oc, ray.direction);
  let c = length_squared(oc) - sphere.radius * sphere.radius;

  let discriminant = half_b * half_b - a * c;
  if discriminant < 0.0 { return false; }
  let sqrtd = sqrt(discriminant);

  // Find the nearest root that lies in the acceptable range
  var root = (-half_b - sqrtd) / a;
  if root <= ray_tmin || ray_tmax <= root {
    root = (-half_b + sqrtd) / a;
    if root <= ray_tmin || ray_tmax <= root {
      return false;
    }
  }

  (*rec).t = root;
  (*rec).point = ray_at(ray, root);
  (*rec).normal = ((*rec).point - sphere.center) / sphere.radius;
  (*rec).material_index = sphere.material_index;

  return true;
}

fn ray_at(ray: Ray, t: f32) -> vec3<f32> {
  return ray.origin + t * ray.direction;
}

fn normal_to_color(normal: vec3<f32>) -> vec3<f32> {
  return (normal + ONE) * 0.5; 
}

// Sets the hit record normal vector
fn set_face_normal(rec: ptr<function, HitRecord>, ray: Ray, outward_normal: vec3<f32>) {
  let front_face = dot(ray.direction, outward_normal) < 0.0;
  (*rec).front_face = front_face;
  if front_face {
    (*rec).normal = outward_normal;
  } else {
    (*rec).normal = -outward_normal;
  }
}


// RANDOM Stuff
var<private> rng: u32;
fn initialize_rng() {
  let u = uv.x;
  let v = uv.y;
  let x = (u << 15u) ^ (u*1729u + 8192374u);
  let y = (v << 21u) ^ (v*271u + 719827371u);
  let t = bitcast<u32>(time) * 87189u + 189273u;
  rng = x ^ y ^ t;
  rand();
}

fn rand() -> f32 {
  rng = rng * 18197u + 182489729u;
  rng ^= (rng << 15u);
  rng ^= (rng >> 22u);
  rng ^= (rng << 4u);

  // Get the random bits such that it defines a float between 1.0 and 2.0
  let randomf_bits = (rng & 0x007fffffu) |  0x3f800000u;
  // return a float between 0.0 and 1.0
  return bitcast<f32>(randomf_bits) - 1.0;
}

fn rand_vec3() -> vec3<f32> {
  return vec3<f32>(rand(), rand(), rand());
}

fn random_in_unit_sphere() -> vec3<f32> {
  loop {
    let p = rand_vec3() * 2.0 - 1.0;
    if length_squared(p) < 1.0 {
      return p;
    }
  }
  // not smart enough to see that it can't make it here
  return ZERO;
}

fn random_unit_vector() -> vec3<f32> {
  return normalize(random_in_unit_sphere());
}

fn random_on_hemisphere(normal: vec3<f32>) -> vec3<f32> {
  let on_unit_sphere = random_unit_vector();
  if (dot(on_unit_sphere, normal) > 0.0) {
    return on_unit_sphere;
  } else {
    return -on_unit_sphere;
  }
}

// Helpers
fn length_squared(x: vec3<f32>) -> f32 {
  let len = length(x);
  return len * len;
}

fn to_vec4(x: vec3<f32>) -> vec4<f32> {
  return vec4<f32> (x.xyz, 1.0);
}
