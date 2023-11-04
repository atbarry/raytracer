use anyhow::Result;
use wgpu::{Device, Features, Instance, Limits, Queue, Surface, SurfaceConfiguration, TextureUsages};
use winit::window::Window;

pub struct RenderEnv {
    pub window: Window,
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface,
    pub surface_config: SurfaceConfiguration,
}

impl RenderEnv {
    pub async fn new(window: Window) -> Result<Self> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Unable to create adapter for surface");

        let trace_path = std::path::Path::new("./trace.txt");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    features: Features::empty(),
                    limits: Limits::default(),
                },
                Some(&trace_path),
            )
            .await?;
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        Ok(Self {
            window,
            surface,
            surface_config,
            device,
            queue,
        })
    }

    pub fn resize(&mut self) {
        let size = self.window.inner_size();
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}
