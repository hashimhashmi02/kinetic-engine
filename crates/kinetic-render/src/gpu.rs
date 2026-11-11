//! Device setup.

use crate::RenderError;

/// Which graphics adapter to ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AdapterChoice {
    /// The fastest adapter available.
    #[default]
    HighPerformance,
    /// A software rasteriser (WARP on Windows, lavapipe on Linux). Slower, but it gives
    /// the same pixels on every machine, which is what golden image tests need.
    Software,
}

impl AdapterChoice {
    /// Reads `KINETIC_ADAPTER`; the value `software` selects [`AdapterChoice::Software`].
    pub fn from_env() -> Self {
        match std::env::var("KINETIC_ADAPTER") {
            Ok(v) if v.eq_ignore_ascii_case("software") => Self::Software,
            _ => Self::HighPerformance,
        }
    }
}

/// A device and its queue, plus a description of the adapter behind them.
#[derive(Debug)]
pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    /// Adapter name and backend, for logs and `kinetic doctor`.
    pub adapter: String,
}

impl Gpu {
    /// Creates a device with no window attached, for offscreen rendering.
    pub fn headless(choice: AdapterChoice) -> Result<Self, RenderError> {
        // Honours WGPU_BACKEND and friends, which helps when one backend misbehaves.
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        pollster::block_on(Self::request(&instance, choice, None))
    }

    /// Creates a device from an existing instance, compatible with `surface` when given.
    pub async fn request(
        instance: &wgpu::Instance,
        choice: AdapterChoice,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self, RenderError> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: choice == AdapterChoice::Software,
                compatible_surface: surface,
                apply_limit_buckets: false,
            })
            .await
            .map_err(|e| RenderError::NoAdapter(e.to_string()))?;
        let info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("kinetic"),
                ..Default::default()
            })
            .await
            .map_err(|e| RenderError::Device(e.to_string()))?;
        Ok(Self { device, queue, adapter: format!("{} ({:?})", info.name, info.backend) })
    }
}
