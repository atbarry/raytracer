use crate::render_env::RenderEnv;
use wgpu::*;
use winit::dpi::PhysicalSize;

#[derive(Debug)]
pub struct Raytracer {
    pipeline: ComputePipeline,
    color_buffer: Texture,
    color_cache: Texture,
    storage_bind_group: BindGroup,
    pub sampler_bind_group: BindGroup,
    pub sampler_bind_layout: BindGroupLayout,
}

impl Raytracer {
    pub fn new(render_env: &RenderEnv, scene_bind_layout: &BindGroupLayout, time_bind_layout: &BindGroupLayout) -> Self {
        let device = &render_env.device;
        let size = render_env.window.inner_size();

        let color_buffer = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: Extent3d {
                width: size.width,
                height: size.height,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        let color_cache = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: Extent3d {
                width: size.width,
                height: size.height,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::COPY_DST
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        let compute_shader =
            device.create_shader_module(include_wgsl!("./shaders/raytrace_kernal.wgsl"));
        let color_buffer_view = color_buffer.create_view(&wgpu::TextureViewDescriptor::default());
        let color_cache_view = color_cache.create_view(&wgpu::TextureViewDescriptor::default());

        let storage_bind_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Ray tracing bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    visibility: ShaderStages::COMPUTE,
                    count: None,
                }],
            });

        let storage_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ray tracing bind group"),
            layout: &storage_bind_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&color_buffer_view),
            }],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });

        let sampler_bind_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Screen bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                        visibility: ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
                        count: None,
                    },
                ],
            });

        let sampler_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Screen bind group"),
            layout: &sampler_bind_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&color_cache_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Raytracer Pipeline Layout"),
            bind_group_layouts: &[
                &storage_bind_layout,
                &sampler_bind_layout,
                scene_bind_layout,
                time_bind_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Ray tracing pipeline"),
            layout: Some(&pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        Self {
            pipeline,
            color_buffer,
            color_cache,
            storage_bind_group,
            sampler_bind_group,
            sampler_bind_layout,
        }
    }

    // pub fn update_resolution(&mut self, device: &Device, resolution: PhysicalSize<u32>) {
    //     self.color_buffer = device.create_texture(&wgpu::TextureDescriptor {
    //         label: None,
    //         size: Extent3d {
    //             width: resolution.width,
    //             height: resolution.height,
    //             ..Default::default()
    //         },
    //         mip_level_count: 1,
    //         sample_count: 1,
    //         dimension: wgpu::TextureDimension::D2,
    //         format: wgpu::TextureFormat::Rgba8Unorm,
    //         usage: TextureUsages::COPY_DST
    //             | TextureUsages::STORAGE_BINDING
    //             | TextureUsages::TEXTURE_BINDING,
    //         view_formats: &[],
    //     });
    // }
    //

    pub fn compute(&self, encoder: &mut CommandEncoder, scene_bind_group: &BindGroup, time_bind_group: &BindGroup) {
        let mut ray_trace_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Ray tracing pass"),
            timestamp_writes: None,
        });

        let width = self.color_buffer.width();
        let height = self.color_buffer.height();

        ray_trace_pass.set_pipeline(&self.pipeline);
        ray_trace_pass.set_bind_group(0, &self.storage_bind_group, &[]);
        ray_trace_pass.set_bind_group(1, &self.sampler_bind_group, &[]);
        ray_trace_pass.set_bind_group(2, scene_bind_group, &[]);
        ray_trace_pass.set_bind_group(3, time_bind_group, &[]);
        ray_trace_pass.dispatch_workgroups(width, height, 1);
        drop(ray_trace_pass);

        encoder.copy_texture_to_texture(
            self.color_buffer.as_image_copy(),
            self.color_cache.as_image_copy(),
            self.color_buffer.size(),
        );
    }
}
