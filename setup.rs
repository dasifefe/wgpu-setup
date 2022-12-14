
pub struct RendererState {
    pub instance: wgpu::Instance,
    /// Device only needs to be wrapped in `Arc` for multi-threaded use.
    pub device: Arc<wgpu::Device>,
    pub queue: wgpu::Queue,
    pub option_target: Option<RendererStateTarget>,
}

pub struct RendererStateBuilderWithTarget<'a> {
    pub limits: wgpu::Limits,
    pub backends: wgpu::Backends,
    pub features: wgpu::Features,
    pub power_preference: wgpu::PowerPreference,
    pub target: RendererStateBuilderTarget<'a>,
}

pub struct RendererStateBuilderTarget<'a> {
    pub window: &'a winit::window::Window,
    pub present_mode: wgpu::PresentMode,
    pub alpha_mode: wgpu::CompositeAlphaMode,
}

impl<'a> TryFrom<RendererStateBuilderWithTarget<'a>> for RendererState {
    type Error = ();
    fn try_from(builder: RendererStateBuilderWithTarget) -> Result<Self, Self::Error> {
        let instance = wgpu::Instance::new(builder.backends);
        let surface = unsafe { instance.create_surface(builder.target.window) };
        let surface_size = builder.target.window.inner_size();
        let adapter = {
            let future_adapter = instance.request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: builder.power_preference,
                    compatible_surface: Option::Some(&surface),
                    force_fallback_adapter: false,
                }
            );
            let Option::Some(adapter) = pollster::block_on(future_adapter) else {
                return Result::Err(());
            };
            adapter
        };
        let (device, queue): (wgpu::Device, wgpu::Queue) = {
            let future_device_queue = adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Option::None,
                    features: builder.features,
                    limits: builder.limits,
                },
                Option::None,
            );
            let Result::Ok((device, queue)) = pollster::block_on(future_device_queue) else {
                return Result::Err(());
            };
            (device, queue)
        };
        let Option::Some(surface_preferred_format) = surface.get_supported_formats(&adapter).get(0).copied() else {
            return Result::Err(());
        };
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_preferred_format,
            width: surface_size.width,
            height: surface_size.height,
            present_mode: builder.target.present_mode,
            alpha_mode: builder.target.alpha_mode,
        };
        surface.configure(&device, &surface_configuration);
        return Result::Ok(
            Self {
                instance: instance,
                device: Arc::new(device),
                queue: queue,
                option_target: Option::Some(
                    RendererStateTarget {
                        surface: surface,
                        configuration: surface_configuration,
                    }
                ),
            }
        );
    }
}
