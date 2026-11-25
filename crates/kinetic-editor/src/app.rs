//! The preview window: the renderer's output with an egui transport bar over it, playing
//! either a fixed timeline or a scene crate that is rebuilt and reloaded on every save.

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use kinetic_core::color::palette;
use kinetic_core::{DVec2, Timeline};
use kinetic_reload::{ReloadEvent, Reloader, SceneCrate};
use kinetic_render::wgpu;
use kinetic_render::{AdapterChoice, Frame, Gpu, MeshCache, Renderer};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::transport::Transport;
use crate::EditorError;

/// How long the "reloaded" note stays on screen.
const TOAST: Duration = Duration::from_secs(2);

/// What the reloader last reported.
#[derive(Debug)]
enum Status {
    /// Nothing to report.
    Ready,
    /// A build is running.
    Building,
    /// The last build failed; the previous timeline, if any, keeps playing.
    Failed(String),
    /// A new timeline arrived.
    Reloaded { took: Duration, at: Instant },
}

impl Status {
    /// Whether the overlay changes over time and needs frames even when paused.
    fn animating(&self) -> bool {
        match self {
            Self::Building => true,
            Self::Reloaded { at, .. } => at.elapsed() < TOAST,
            _ => false,
        }
    }
}

struct View {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    gpu: Gpu,
    renderer: Renderer,
    /// Meshes from the last frame; nodes whose geometry did not change reuse them.
    meshes: MeshCache,
    egui: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
}

struct App {
    instance: wgpu::Instance,
    title: String,
    timeline: Option<Timeline>,
    transport: Transport,
    status: Status,
    view: Option<View>,
    error: Option<EditorError>,
    /// Kept alive so the scene keeps being watched; dropped when the window closes.
    _reloader: Option<Reloader>,
}

impl View {
    fn new(
        event_loop: &ActiveEventLoop,
        instance: &wgpu::Instance,
        title: &str,
    ) -> Result<Self, EditorError> {
        let mut attributes = Window::default_attributes().with_title(title);
        // 70% of the screen, 16:9, centred. A fixed logical size could fill a scaled laptop
        // screen entirely and push the title bar, with its minimize button, off screen.
        match event_loop.primary_monitor() {
            Some(monitor) => {
                let (screen, origin) = (monitor.size(), monitor.position());
                let width = (f64::from(screen.width) * 0.7)
                    .min(f64::from(screen.height) * 0.7 * 16.0 / 9.0);
                let size = PhysicalSize::new(width as u32, (width * 9.0 / 16.0) as u32);
                let x = origin.x + (screen.width.saturating_sub(size.width) / 2) as i32;
                let y = origin.y + (screen.height.saturating_sub(size.height) / 2) as i32;
                attributes =
                    attributes.with_inner_size(size).with_position(PhysicalPosition::new(x, y));
            }
            None => attributes = attributes.with_inner_size(LogicalSize::new(960.0, 540.0)),
        }
        let window = Arc::new(
            event_loop.create_window(attributes).map_err(|e| EditorError::Window(e.to_string()))?,
        );
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| EditorError::Window(e.to_string()))?;
        let gpu =
            pollster::block_on(Gpu::request(instance, AdapterChoice::from_env(), Some(&surface)))
                .map_err(EditorError::Render)?;

        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&gpu.adapter, size.width.max(1), size.height.max(1))
            .ok_or_else(|| EditorError::Window("surface is not supported by the adapter".into()))?;
        config.present_mode = wgpu::PresentMode::AutoVsync;
        surface.configure(&gpu.device, &config);

        let renderer = Renderer::new(&gpu.device, config.format);
        let context = egui::Context::default();
        let egui = egui_winit::State::new(
            context,
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            window.theme(),
            Some(gpu.device.limits().max_texture_dimension_2d as usize),
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &gpu.device,
            config.format,
            egui_wgpu::RendererOptions::default(),
        );
        let meshes = MeshCache::new();
        Ok(Self { window, surface, config, gpu, renderer, meshes, egui, egui_renderer })
    }
}

impl ApplicationHandler<ReloadEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.view.is_none() {
            match View::new(event_loop, &self.instance, &self.title) {
                Ok(view) => self.view = Some(view),
                Err(error) => {
                    self.error = Some(error);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(view) = self.view.as_mut() else { return };
        let response = view.egui.on_window_event(&view.window, &event);
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                view.config.width = size.width.max(1);
                view.config.height = size.height.max(1);
                view.surface.configure(&view.gpu.device, &view.config);
                view.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.transport.tick();
                view.redraw(self.timeline.as_ref(), &mut self.transport, &self.status);
                if self.transport.playing || self.status.animating() {
                    view.window.request_redraw();
                }
            }
            _ if response.repaint => view.window.request_redraw(),
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: ReloadEvent) {
        match event {
            ReloadEvent::Building => self.status = Status::Building,
            ReloadEvent::Loaded { timeline, took } => {
                eprintln!("kinetic: scene loaded in {:.2} s", took.as_secs_f64());
                self.transport.retime(timeline.duration);
                self.timeline = Some(*timeline);
                self.status = Status::Reloaded { took, at: Instant::now() };
            }
            ReloadEvent::Failed(text) => {
                eprintln!("kinetic: build failed, the previous version keeps playing\n{text}");
                self.status = Status::Failed(text);
            }
        }
        if let Some(view) = &self.view {
            view.window.request_redraw();
        }
    }
}

/// Draws the reloader's status in the top-left corner.
fn status_overlay(ctx: &egui::Context, status: &Status, has_timeline: bool) {
    let area = egui::Area::new(egui::Id::unique("kinetic-status"))
        .anchor(egui::Align2::LEFT_TOP, [12.0, 12.0])
        .interactable(true);
    match status {
        Status::Ready => {}
        Status::Building => {
            area.show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(if has_timeline {
                            "Rebuilding the scene"
                        } else {
                            "Building the scene. The first build compiles its dependencies \
                             and can take a minute."
                        });
                    });
                });
            });
        }
        Status::Failed(text) => {
            area.show(ctx, |ui| {
                egui::Frame::popup(ui.style()).fill(egui::Color32::from_rgb(52, 16, 22)).show(
                    ui,
                    |ui| {
                        ui.set_max_width(760.0);
                        let headline = if has_timeline {
                            "Build failed. The previous version keeps playing."
                        } else {
                            "Build failed."
                        };
                        ui.label(
                            egui::RichText::new(headline)
                                .strong()
                                .color(egui::Color32::from_rgb(255, 176, 166)),
                        );
                        egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                            ui.label(egui::RichText::new(text).monospace());
                        });
                    },
                );
            });
        }
        Status::Reloaded { took, at } => {
            if at.elapsed() < TOAST {
                area.show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.label(format!("Reloaded in {:.2} s", took.as_secs_f64()));
                    });
                });
            }
        }
    }
}

impl View {
    fn redraw(&mut self, timeline: Option<&Timeline>, transport: &mut Transport, status: &Status) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            // Minimised, resized mid-frame or lost: reconfigure and try on the next redraw.
            _ => {
                self.surface.configure(&self.gpu.device, &self.config);
                return;
            }
        };
        let target = frame.texture.create_view(&Default::default());
        let size = (self.config.width, self.config.height);

        // Transport bar and status overlay.
        let input = self.egui.take_egui_input(&self.window);
        let context = self.egui.egui_ctx().clone();
        let output = context.run_ui(input, |ui| {
            egui::Panel::bottom("transport").show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(if transport.playing { "Pause" } else { "Play" }).clicked() {
                        transport.playing = !transport.playing;
                    }
                    ui.spacing_mut().slider_width = (ui.available_width() - 90.0).max(80.0);
                    let slider = egui::Slider::new(&mut transport.time, 0.0..=transport.duration)
                        .suffix(" s")
                        .fixed_decimals(2);
                    if ui.add(slider).dragged() {
                        // Scrubbing takes over from playback.
                        transport.playing = false;
                    }
                });
            });
            status_overlay(ui.ctx(), status, timeline.is_some());
        });
        self.egui.handle_platform_output(&self.window, output.platform_output);
        let jobs = context.tessellate(output.shapes, output.pixels_per_point);
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [size.0, size.1],
            pixels_per_point: output.pixels_per_point,
        };

        let (device, queue) = (&self.gpu.device, &self.gpu.queue);
        let mut encoder = device.create_command_encoder(&Default::default());

        let mut scene = match timeline {
            Some(timeline) => {
                let state = timeline.evaluate(transport.time);
                // Flatten curves to a quarter of a pixel at the current window size.
                let pixels_per_unit = (f64::from(size.0) / state.size.x)
                    .min(f64::from(size.1) / state.size.y)
                    .max(1e-6);
                self.meshes.frame(&state, 0.25 / pixels_per_unit)
            }
            // Nothing built yet: an empty stage.
            None => Frame {
                background: palette::ink(),
                size: DVec2::new(1280.0, 720.0),
                items: Vec::new(),
            },
        };
        // The scene keeps its aspect ratio: fit the scene rectangle inside the window.
        let window_aspect = f64::from(size.0) / f64::from(size.1);
        let scene_aspect = scene.size.x / scene.size.y;
        if window_aspect > scene_aspect {
            scene.size.x = scene.size.y * window_aspect;
        } else {
            scene.size.y = scene.size.x / window_aspect;
        }
        self.renderer.draw(device, queue, &mut encoder, &target, size, &scene);

        // egui insists every texture change is consumed, so drain rather than borrow.
        let mut textures = output.textures_delta;
        for (id, deltas) in textures.set.drain() {
            for delta in &deltas {
                self.egui_renderer.update_texture(device, queue, id, delta);
            }
        }
        let extra = self.egui_renderer.update_buffers(device, queue, &mut encoder, &jobs, &screen);
        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("kinetic ui"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
                .forget_lifetime();
            self.egui_renderer.render(&mut pass, &jobs, &screen);
        }
        for id in textures.free.drain() {
            self.egui_renderer.free_texture(&id);
        }
        queue.submit(extra.into_iter().chain([encoder.finish()]));
        self.window.pre_present_notify();
        queue.present(frame);
    }
}

fn event_loop() -> Result<EventLoop<ReloadEvent>, EditorError> {
    let event_loop = EventLoop::<ReloadEvent>::with_user_event()
        .build()
        .map_err(|e| EditorError::Window(e.to_string()))?;
    event_loop.set_control_flow(ControlFlow::Wait);
    Ok(event_loop)
}

fn drive(event_loop: EventLoop<ReloadEvent>, mut app: App) -> Result<(), EditorError> {
    event_loop.run_app(&mut app).map_err(|e| EditorError::Window(e.to_string()))?;
    app.error.map_or(Ok(()), Err)
}

fn instance(event_loop: &EventLoop<ReloadEvent>) -> wgpu::Instance {
    let mut descriptor = wgpu::InstanceDescriptor::new_with_display_handle_from_env(Box::new(
        event_loop.owned_display_handle(),
    ));
    // On Windows the window presents through DX12 unless WGPU_BACKEND says otherwise: a
    // Vulkan swapchain on a laptop with switchable graphics made the whole screen flicker.
    if cfg!(windows) && std::env::var_os("WGPU_BACKEND").is_none() {
        descriptor.backends = wgpu::Backends::DX12;
    }
    wgpu::Instance::new(descriptor)
}

/// Opens a preview window playing `timeline` on a loop and runs until it is closed.
pub fn run(timeline: Timeline) -> Result<(), EditorError> {
    let event_loop = event_loop()?;
    let app = App {
        instance: instance(&event_loop),
        title: "Kinetic".into(),
        transport: Transport::new(timeline.duration),
        timeline: Some(timeline),
        status: Status::Ready,
        view: None,
        error: None,
        _reloader: None,
    };
    drive(event_loop, app)
}

/// Opens a preview window on the scene crate in `dir`, and rebuilds and reloads it every
/// time one of its files is saved. Runs until the window is closed.
pub fn run_scene(dir: &Path) -> Result<(), EditorError> {
    let scene = SceneCrate::open(dir).map_err(|e| EditorError::Scene(e.to_string()))?;
    let event_loop = event_loop()?;
    let proxy = event_loop.create_proxy();
    let scratch = std::env::temp_dir().join("kinetic").join("scratch");
    let reloader =
        Reloader::start(scene, SceneCrate::default_target_dir(), scratch, move |event| {
            // Only fails once the window has closed, when nobody is listening.
            let _ = proxy.send_event(event);
        })
        .map_err(|e| EditorError::Scene(e.to_string()))?;
    let name = dir
        .canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| dir.display().to_string());
    let app = App {
        instance: instance(&event_loop),
        title: format!("Kinetic: {name}"),
        timeline: None,
        transport: Transport::new(0.0),
        status: Status::Building,
        view: None,
        error: None,
        _reloader: Some(reloader),
    };
    drive(event_loop, app)
}
