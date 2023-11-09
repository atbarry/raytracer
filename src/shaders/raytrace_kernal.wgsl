struct Sphere {
  color: vec3<f32>,
  center: vec3<f32>,
  radius: f32,
}

struct Ray {
  origin: vec3<f32>,
  direction: vec3<f32>,
}

struct HitRecord {
  point: vec3<f32>,
  normal: vec3<f32>,
  color: vec3<f32>,
  t: f32,
  front_face: bool,
}

struct Camera {
  pos: vec3<f32>,
  forward: vec3<f32>,
  right: vec3<f32>,
  up: vec3<f32>,
  focal_length: f32,
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
const RAY_TMAX: f32 = 1000.0;
const RAY_TMIN: f32 = 0.5;
const SAMPLES_PER_PIXEL: u32 = 1u;
const MAX_CACHE_WEIGHT: f32 = 15.0 / 16.0;

// Other constants
const PI: f32 = 3.1415926535897932385;
const MAX_U32: u32 = 4294967295u;

@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;
@group(1) @binding(0) var color_cache: texture_2d<f32>;
@group(1) @binding(1) var csampler: sampler;
@group(2) @binding(0) var<uniform> camera: Camera;
@group(2) @binding(1) var<storage, read> objects: ObjectData;
@group(2) @binding(2) var<uniform> cached_frames: i32;
@group(3) @binding(0) var<uniform> time: f32;

@compute @workgroup_size(1,1,1)
fn main(@builtin(global_invocation_id) uv: vec3<u32>) {
  initialize_rng(uv.x, uv.y);
  let screen_size = textureDimensions(color_buffer);
  let scan_x = f32(uv.x);
  let scan_y = f32(uv.y);
  let scan = vec2<f32>(scan_x, scan_y);

  let viewport_height = 2.0;
  let viewport_width = viewport_height * f32(screen_size.x) / f32(screen_size.y);

  let viewport_u = camera.right * viewport_width;
  let viewport_v = -camera.up * viewport_height;
  
  let pixel_delta_u = viewport_u / f32(screen_size.x);
  let pixel_delta_v = viewport_v / f32(screen_size.y);

  let viewport_bottom_left = camera.pos + camera.forward*camera.focal_length - viewport_u / 2.0 - viewport_v / 2.0;
  let pixel00_pos = viewport_bottom_left + 0.5 * (pixel_delta_u + pixel_delta_v);

  let pixel_center = pixel00_pos + scan_x*pixel_delta_u + scan_y*pixel_delta_v;
  let pixel_color = send_rays(pixel_center, pixel_delta_v + pixel_delta_u);

  store_color(color_ajustments(pixel_color), uv.xy);
}

fn store_color(pixel_color: vec4<f32>, uv: vec2<u32>) {
  let cached_color = textureLoad(color_cache, uv, 0);

  let frames = f32(cached_frames + 1);
  var cw = f32(cached_frames) / frames;

  if cw > MAX_CACHE_WEIGHT {
    cw = MAX_CACHE_WEIGHT;
  }

  let rw = 1.0 - cw;
  let color = pixel_color*rw + cached_color*cw; 
  textureStore(color_buffer, uv, color);
}

fn send_rays(pixel_center: vec3<f32>, pixel_delta: vec3<f32>) -> vec3<f32> {
  var color = ZERO;
  for(var sample = 0u; sample < SAMPLES_PER_PIXEL; sample++) {
    let ray = get_random_ray(pixel_center, pixel_delta, sample);
    color += ray_color(ray);
  }
  return color / f32(SAMPLES_PER_PIXEL);
}

fn ray_color(ray: Ray) -> vec3<f32> {
  var rec: HitRecord;

  if hit(ray, &rec) {
    return normal_to_color(rec.normal);
    // return  rec.color * dot(rec.color, rec.normal);
  }

  let a = 0.5 * (ray.direction.y + 1.0);
  // let color = (1.0 - a) * ONE + a * vec3(0.5, 0.7, 1.0);
  let color = ZERO;
  return color;
}

fn get_random_ray(pixel_center: vec3<f32>, pixel_delta: vec3<f32>, ray_index: u32) -> Ray {
  let px = -0.5 + rand();
  let py = -0.5 + rand();
  let pixel_sample = pixel_center + vec3<f32>(px, py, 1.0) * pixel_delta;

  return Ray(camera.pos, pixel_sample);
}

fn hit(ray: Ray, rec: ptr<function, HitRecord>) -> bool {
  var temp_rec: HitRecord;
  var hit_anything = false;
  var closest_so_far = RAY_TMAX;

  for (var i = 0u; i < arrayLength(&objects.spheres); i++) {
    let sphere = objects.spheres[i];
    if hit_sphere(ray, sphere, RAY_TMIN, closest_so_far, &temp_rec) {
      hit_anything = true;
      closest_so_far = temp_rec.t;
      *rec = temp_rec;
    }
  }

  return hit_anything;
}

fn hit_sphere(
  ray: Ray, 
  sphere: Sphere, 
  ray_tmin: f32, 
  ray_tmax: f32, 
  rec: ptr<function, HitRecord>
) -> bool {
  let oc = ray.origin - sphere.center;
  let a = pow(length(ray.direction), 2.0);
  let half_b = dot(oc, ray.direction);
  let c = pow(length(oc), 2.0) - sphere.radius * sphere.radius;

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
  (*rec).color = sphere.color;

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

fn color_ajustments(color: vec3<f32>) -> vec4<f32> {
  let uniformish = pow(color, ONE * 2.2);
  return vec4<f32>(uniformish.xyz, 1.0);
}

var<private> rng: u32;
fn initialize_rng(u: u32, v: u32) {
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
